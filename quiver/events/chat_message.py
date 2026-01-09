import json
import config as e

from rabbitmq.rabbitmq_conn import pikaConn
channel = pikaConn.channel
# channel.queue_declare(queue='twitch_chat')
# channel.queue_bind(
#     exchange=e.exchange_name, 
#     queue='twitch_chat',
#     arguments={
#         'x-message-ttl': 10000 # in milliseconds
#     }
# )

# callback functions
def on_message(notification):
    payload = notification
    subscription = payload["subscription"]
    event = payload["event"]

    # create json object and send to rabbitmq
    rmq_json = {
        "id": subscription["id"],
        "broadcaster_user_id": event["broadcaster_user_id"],
        "broadcaster_user_name": event["broadcaster_user_name"],
        "chatter_user_id": event["chatter_user_id"],
        "chatter_user_name": event["chatter_user_name"],
        "message_text": event["message"]["text"],
        "badges": event["badges"]
    }
    # print(rmq_json)

    rmq_msgReq = { 
      "method": "chatcompletion", 
      "replyTo": ["testchannel"],
      "params": { 
        "role": "user", 
        "content": f'[TWITCH CHAT] {event["chatter_user_name"]} said: {event["message"]["text"]}'
      }
    }

    properties = pika.BasicProperties(
    expiration='5000'  # Time in milliseconds as a string
    )

    # publish to rabbitmq queue
    channel.basic_publish(
        exchange=e.exchange_name, 
        routing_key='ai.chatcompletion', 
        body=json.dumps(rmq_msgReq),
        properties=properties
    )
    print("message published")

# {
#   "subscription": {
#     "id": "0b7f3361-672b-4d39-b307-dd5b576c9b27",
#     "status": "enabled",
#     "type": "channel.chat.message",
#     "version": "1",
#     "condition": {
#       "broadcaster_user_id": "1971641",
#       "user_id": "2914196"
#     },
#     "transport": {
#       "method": "websocket",
#       "session_id": "AgoQHR3s6Mb4T8GFB1l3DlPfiRIGY2VsbC1h"
#     },
#     "created_at": "2023-11-06T18:11:47.492253549Z",
#     "cost": 0
#   },
#   "event": {
#     "broadcaster_user_id": "1971641",
#     "broadcaster_user_login": "streamer",
#     "broadcaster_user_name": "streamer",
#     "chatter_user_id": "4145994",
#     "chatter_user_login": "viewer32",
#     "chatter_user_name": "viewer32",
#     "message_id": "cc106a89-1814-919d-454c-f4f2f970aae7",
#     "message": {
#       "text": "Hi chat",
#       "fragments": [
#         {
#           "type": "text",
#           "text": "Hi chat",
#           "cheermote": null,
#           "emote": null,
#           "mention": null
#         }
#       ]
#     },
#     "color": "#00FF7F",
#     "badges": [
#       {
#         "set_id": "moderator",
#         "id": "1",
#         "info": ""
#       },
#       {
#         "set_id": "subscriber",
#         "id": "12",
#         "info": "16"
#       },
#       {
#         "set_id": "sub-gifter",
#         "id": "1",
#         "info": ""
#       }
#     ],
#     "message_type": "text",
#     "cheer": null,
#     "reply": null,
#     "channel_points_custom_reward_id": null,
#     "source_broadcaster_user_id": null,
#     "source_broadcaster_user_login": null,
#     "source_broadcaster_user_name": null,
#     "source_message_id": null,
#     "source_badges": null
#   }
# }