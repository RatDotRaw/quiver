use tiny_http::{Response, Server};
use tokio::sync::oneshot;

use url::Url;

use crate::twitch::eventsub_types::EventSubType;

pub struct TwitchAuthenticator {
    /// Twitch app client ID
    pub client_id: String,
    /// client secret for OAuth
    client_secret: String,
    /// User access token, used as the Bearer token in API requests
    pub access_token: Option<String>,
    refresh_token: Option<String>,
    expires_in: Option<u64>,
    redirect_uri: String,
    pub user_data: Option<TwitchUserData>,
}

#[derive(serde::Deserialize)]
pub struct TwitchUserData {
    pub id: String,
    pub login: String,
    pub display_name: String,
}

#[derive(Debug, serde::Deserialize)]
struct TwitchOauthTokenResponse {
    access_token: String,
    refresh_token: String,
    expires_in: i64,
}

#[derive(serde::Deserialize)]
struct TwitchUsersResponse {
    data: [TwitchUserData; 1],
}

impl TwitchAuthenticator {
    pub fn new(client_id: String, client_secret: String) -> Self {
        Self {
            client_id,
            client_secret,
            access_token: None,
            refresh_token: None,
            expires_in: None,
            redirect_uri: String::from("http://localhost:5000/auth"),
            user_data: None,
        }
    }

    pub async fn authenticate(&mut self, subscription_types: &[EventSubType]) -> Result<(), std::io::Error> {
        let (tx, rx) = oneshot::channel::<String>();

        // start small http server for listening to the auth code.
        tokio::spawn(async move {
            let server = Server::http("127.0.0.1:5000").unwrap();
            for request in server.incoming_requests() {
                if request.url().contains("code=") {
                    println!("Auth code received!");
                    let url =
                        Url::parse(&("http://local.host".to_owned() + &request.url())).unwrap();
                    if let Some((_, val)) = url.query_pairs().find(|(key, _)| key == "code") {
                        request
                            .respond(Response::from_string(
                                "Authentication Done!\nYou may close this page.",
                            ))
                            .expect("Failed to send response");
                        tx.send(val.to_string()).unwrap();
                        break;
                    }
                };
                request.respond(Response::from_string("Either something went wrong or you are lost.\nFollow authentication link in the console.")).expect("Failed to send response");
            }
        });

        // resolve subscription types to scopes
        let scopes = EventSubType::resolve_scopes(subscription_types);

        // construct Oauth2 url for user
        let mut url = Url::parse("https://id.twitch.tv/oauth2/authorize").unwrap();
        {
            let mut query = url.query_pairs_mut();
            query
                .append_pair("client_id", &self.client_id)
                .append_pair("redirect_uri", &self.redirect_uri)
                .append_pair("response_type", "code");

            query.append_pair("scope", &scopes.join(" "));
            // for scope in scopes {
            //     query.append_pair("scope", scope);
            // }
        }

        println!(
            "### AUTH REQUIRED ###\nOpen this URL to complete authentication: \n{}\n### #### ######## ###",
            url.as_str()
        );

        let authorization_code = rx.await.unwrap();
        println!("authorization code: {}", authorization_code);

        // make api req to get access_token & refres_token
        let form_params = [
            ("client_id", &self.client_id),
            ("client_secret", &self.client_secret),
            ("code", &authorization_code.parse().unwrap()),
            ("grant_type", &String::from("authorization_code")),
            ("redirect_uri", &self.redirect_uri),
        ];

        let client = reqwest::Client::new();
        let resp = client
            .post("https://id.twitch.tv/oauth2/token")
            .form(&form_params)
            .send()
            .await
            .expect("Requesting access_token failed");

        let code = resp
            .json::<TwitchOauthTokenResponse>()
            .await
            .expect("Failed to decerialize response");

        self.access_token = Some(code.access_token);
        self.refresh_token = Some(code.refresh_token);
        self.expires_in = Some(code.expires_in as u64);

        println!("access token received and assigned!");

        self.get_user_data().await.unwrap();

        Ok(())
    }

    /// Get the current user id for later use.
    /// We dont need the rest.
    /// Docs: https://dev.twitch.tv/docs/api/reference/#get-users
    pub async fn get_user_data(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let access_token = self
            .access_token
            .as_ref()
            .ok_or("No access token available. Run authenticate() first.")?;

        let client = reqwest::Client::new();
        let response = client
            .get("https://api.twitch.tv/helix/users")
            .header("Client-Id", &self.client_id)
            .header("Authorization", &("Bearer ".to_owned() + access_token))
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await?;
            return Err(format!("Failed to fetch user data ({}): {}", status, body).into());
        }

        let response = response.json::<TwitchUsersResponse>().await?;
        let user_data = response
            .data
            .into_iter()
            .next()
            .ok_or("User data array was empty")?;

        self.user_data = Some(user_data);
        println!(
            "User data received and assigned! User's id is: {}",
            self.user_data.as_ref().unwrap().id
        );

        return Ok(());
    }
}
