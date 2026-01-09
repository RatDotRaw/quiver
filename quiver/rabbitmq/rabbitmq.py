import pika
import logging
from threading import Lock

class RabbitMQConnection():
    """A singleton class to hold the pika connection"""

    _instance = None
    _lock = Lock()
    
    def __new__(cls, *args, **kwargs):
        with cls._lock:
            if cls._instance is None:
                cls._instance = super().__new__(cls)
        return cls._instance
            
    def __init__(self, host="localhost", port=5672, username=None, password=None):
        # Prevent re-initialization of the Singleton attributes
        if not hasattr(self, '_initialized'):
            self.host = host
            self.port = port
            self.username = username
            self.password = password
            
            self.connection = None
            self.channel = None
            self._initialized = True # Set the flag
    
    def connect(self):
        """Establish a connection to RabbitMQ."""
        if self.connection and self.connection.is_open:
            return
        if self.connection and self.connection.is_closed:
            self.connection = None
            self.channel = None

        try:
            credentials = None
            if self.username and self.password:
                credentials = pika.PlainCredentials(self.username, self.password)

            parameters = pika.ConnectionParameters(
                host=self.host,
                port=self.port,
                credentials=credentials,
                heartbeat=600,
                # connection_timeout=15
            )

            self.connection = pika.BlockingConnection(parameters)
            self.channel = self.connection.channel()
        except pika.exceptions.AMQPError as e:
            logging.error(f'[PIKACONN]: Failed to connect to RabbitMQ: {str(e)}')
            raise 
        print("[PIKACONN]: Connected to rabbitMQ")

    def get_channel(self):
        """Get the RabbitMQ channel, ensuring the connection is active."""
        if not (self.channel and self.channel.is_open):
            self.connect() # Attempt to connect/reconnect
        if not self.channel:
            logging.error('Connection not established. Connect first.')
            raise 
        return self.channel
    
    def close(self):
        """Close the connection to RabbitMQ."""
        if self.channel:
            self.channel.close()
        if self.connection:
            self.connection.close()
        self.connection = None
        self.channel = None
            