import threading
import time
import urllib.parse
import webbrowser
import logging

import requests
from flask import Flask
from flask import request

class TwitchAuthenticator:
    """Handles Twitch OAuth authentication flow."""
    
    TWITCH_AUTH_URL = "https://id.twitch.tv/oauth2/authorize"
    TWITCH_TOKEN_URL = "https://id.twitch.tv/oauth2/token"
    TWITCH_API_URL = "https://api.twitch.tv/helix"
    USER_DATA_URL = "https://api.twitch.tv/helix/users"
    REDIRECT_URI = "http://localhost:5000"

    def __init__(self, client_id: str, client_secret: str):
        """Initialize the authenticator with client credentials.
        
        Args:
            client_id: Twitch client ID
            client_secret: Twitch client secret
        """
        self.client_id = client_id
        self.client_secret = client_secret

        self.user_id: int = None

        self._access_token = None
        self._refresh_token = None
        self._expires_in = None

    def get_access_token(self, scopes: list[str]) -> dict[str, str]:
        """Perform the OAuth flow to get access token and user data.
        
        Args:
            scopes: List of Twitch API scopes to request
            
        Returns:
            Dictionary containing access_token, refresh_token, and user info
        """
        result = {"ready": False}
        app = Flask(__name__)
        
        # Define a route for handling the authorization callback
        @app.route("/")
        def auth_callback():
            # Check if the authorization code is present in the request arguments
            auth_code = request.args.get('code')
            if auth_code is None:
                return "No authorization code received", 400

            # exchange the authorization code for an access token
            params = {
                "client_id": self.client_id,
                "client_secret": self.client_secret,
                "code": auth_code,
                "grant_type": "authorization_code",
                "redirect_uri": self.REDIRECT_URI # ensure this matches the redirect URI in the Twitch app settings
            }
            response = requests.post(self.TWITCH_TOKEN_URL, data=params)
            response = response.json()

            if not "access_token" in response and not "refresh_token" in response:
                raise Exception("Failed to exchange authorization code for tokens")
            
            # print(response) # don't leave this uncommented
        
            self._access_token = response["access_token"]
            self._refresh_token = response["refresh_token"]
            self._expires_in = response["expires_in"]

            result["access_token"] = response["access_token"]
            result["refresh_token"] = response["refresh_token"]
            result["expires_in"] = response["expires_in"]
            result["ready"] = True
            return """
                <!DOCTYPE html>
                <html>
                <head>
                    <title>Authentication Successful</title>
                </head>
                <body>
                    <h2>Authentication completed successfully!</h2>
                    <p>This window will close automatically...</p>
                    <script>
                        window.close();
                    </script>
                </body>
                </html>
                """, 200
        
        # Start Flask server in a background thread
        flask_thread = threading.Thread(
            target=app.run, 
            daemon=True, 
            name="TwitchAuthServer",
            kwargs={"port": 5000, "debug": False}
        )
        flask_thread.start()

        # build and open login url with scopes
        params = {
            "client_id": self.client_id,
            "redirect_uri": self.REDIRECT_URI,
            "response_type": "code",
            "scope": " ".join(scopes)
        }
        auth_url = f"{self.TWITCH_AUTH_URL}?{urllib.parse.urlencode(params)}"
        print(f"Opening login URL: {auth_url}")
        webbrowser.open(auth_url)

        # Wait for the authentication to complete
        while not result.get("ready", False):
            time.sleep(0.2)

        self.fetch_user_data()

        # Clean up the result before returning
        result.pop("ready", None)
        return result

    def fetch_user_data(self):
        """Fetch user data from Twitch API.

        Returns:
            Dictionary containing user data
        """
        # check if access token exists
        if not self._access_token:
            raise Exception("Access token not available")
        
        headers = {
            "Client-ID": self.client_id,
            "Authorization": f"Bearer {self._access_token}"
        }
        response = requests.get(self.USER_DATA_URL, headers=headers)
        
        if response.status_code == 200:
            response = response.json()

            # {'data': [{'id': '182465973', 'login': 'ratdotraw', 'display_name': 'ratdotraw'}]}
            self.user_id = response["data"][0]["id"]
            print(f"user id set to:{self.user_id}")
            
            return response
        else:
            raise Exception(f"Failed to fetch user data: {response.status_code} - {response.text}")

    def refresh_token(self, refresh_token: str) -> tuple[str, str]:
        """Refresh an expired access token.
        
        Args:
            refresh_token: The refresh token to use
            
        Returns:
            Tuple of (new_access_token, new_refresh_token)
            
        Raises:
            InvalidTokenError: If refresh fails due to invalid credentials
        """
        try:
            params = {
                "client_id": self.client_id,
                "client_secret": self.client_secret,
                "refresh_token": refresh_token,
                "grant_type": "refresh_token"
            }
            
            response = requests.post(self.TWITCH_TOKEN_URL, params=params)
            token_data = response.json()
            
            if "access_token" not in token_data or "refresh_token" not in token_data:
                error_msg = token_data.get("message", "Unknown error")
                raise Exception(f"Failed to refresh token: {error_msg}")
                
            return token_data["access_token"], token_data["refresh_token"]

        except requests.ConnectionError:
            raise Exception("Network error while refreshing token")