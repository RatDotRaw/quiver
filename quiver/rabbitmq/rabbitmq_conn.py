import config as e
from .rabbitmq import RabbitMQConnection

pikaConn = RabbitMQConnection(
    host=e.exchange_host,
    port=e.exchange_port,
    username=e.exchange_user, 
    password=e.exchange_pass
)

pikaConn.connect()