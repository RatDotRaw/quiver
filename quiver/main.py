import config
# import events_config
from twitch.authentication import TwitchAuthenticator
from twitch.twitch_event_sub import TwitchEventSub
# for a list of scopes, read https://dev.twitch.tv/docs/authentication/scopes/#twitch-access-token-scopes
# for a list of event types, read https://dev.twitch.tv/docs/eventsub/eventsub-subscription-types/
# 

# channel = pikaConn.channel

auth = TwitchAuthenticator(config.client_id, config.client_secret)
result = auth.get_access_token(scopes=config.scopes)
# result2 = auth.fetch_user_data()
# print(result)
# print(result2)

# import all events callback functions
from events import on_message

# 75738685 # insym
# 151368796 # piratesoftware
# 552120296 # zackrawrr
# 79792848 # squchan
# 1150749706 # miatr 

sub = TwitchEventSub(
    client_id=config.client_id,
    access_token=result["access_token"],
    broadcaster_id="79792848",
    moderator_user_id=auth.user_id,
    handlers={
        "channel.chat.message": {
            "version": 1,
            "callback": on_message
        }
    }
)

# some logging
print("exchange name:", config.exchange_name)

sub.connect()