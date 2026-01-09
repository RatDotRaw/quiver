import config as e

# channel = pikaConn.channel
# channel.queue_declare(queue='twitch_chat')
# channel.queue_bind(
#     exchange=e.exchange_name, 
#     queue='twitch_chat',
#     # arguments={
#     #     'x-message-ttl': 10000 # in milliseconds
#     # }
# )

# callback functions
def on_follow(notification):
    payload = notification["payload"]
    subscription = payload["subscription"]
    event = payload["event"]
    
    # create json object and send to rabbitmq
    rmq_json = {
        "broadcaster_user_id": event["broadcaster_user_id"],
        "broadcaster_user_name": event["broadcaster_user_name"],
        "user_id": event["user_id"],
        "user_name": event["user_name"],
        "followed_at": event["followed_at"],
    }

    # publish to rabbitmq queue
    # channel.basic_publish(
    #     exchange=e.exchange_name, 
    #     routing_key='channel.follow', 
    #     body=str(rmq_json)
    # )

# {
#     "subscription": {
#         "id": "f1c2a387-161a-49f9-a165-0f21d7a4e1c4",
#         "type": "channel.follow",
#         "version": "2",
#         "status": "enabled",
#         "cost": 0,
#         "condition": {
#            "broadcaster_user_id": "1337",
#            "moderator_user_id": "1337"
#         },
#          "transport": {
#             "method": "webhook",
#             "callback": "https://example.com/webhooks/callback"
#         },
#         "created_at": "2019-11-16T10:11:12.634234626Z"
#     },
#     "event": {
#         "user_id": "1234",
#         "user_login": "cool_user",
#         "user_name": "Cool_User",
#         "broadcaster_user_id": "1337",
#         "broadcaster_user_login": "cooler_user",
#         "broadcaster_user_name": "Cooler_User",
#         "followed_at": "2020-07-15T18:16:11.17106713Z"
#     }
# }