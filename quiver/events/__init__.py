from rabbitmq.rabbitmq_conn import RabbitMQConnection
# import config as e

# pikaConn = RabbitMQConnection(username="user", password="changeme")
# pikaConn.connect()

# channel = pikaConn.get_channel()

from .follow import on_follow
from .chat_message import on_message