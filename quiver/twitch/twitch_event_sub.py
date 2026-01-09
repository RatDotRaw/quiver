import logging
import asyncio
import websockets
import json
import typing
import requests

class TwitchEventSub():
    """
    Twitch Event Sub client for subscribing to events on Twitch channels.
    """

    WEBSOCKET_URL: str = 'wss://eventsub.wss.twitch.tv/ws'
    API_URL: str = "https://api.twitch.tv/helix"
    
    def __init__(
        self, 
        client_id: str, 
        access_token: str,
        broadcaster_id: int,
        moderator_user_id: int,
        handlers: dict
    ):
        self.client_id: str = client_id
        self._access_token: str = access_token
        self.broadcaster_id: int = broadcaster_id
        self.moderator_user_id: int = moderator_user_id
        self._handlers = handlers if handlers is not None else {}

        self._websocket = None
        self._session_id = None

        # Set up the logger
        self._logger = logging.getLogger(__name__)
        self._logger.setLevel(logging.ERROR)
        # Create a console handler and set the level to DEBUG
        ch = logging.StreamHandler()
        ch.setLevel(logging.DEBUG)
        # Create a formatter and add it to the handler
        formatter = logging.Formatter('%(asctime)s - %(name)s - %(levelname)s - %(message)s')
        ch.setFormatter(formatter)
        # Add the handler to the logger
        self._logger.addHandler(ch)
    
    def connect(self) -> None:
        self._logger.info("Starting twitch event sub...")

        if self._access_token is None:
            raise Exception("Access token not set")
        
        asyncio.run(self._start_websocket())
        
    def _handle_event(self, event_type, payload: dict[str, any]):
        """Dispatch event to the approriate handler"""
        event_details = self._handlers.get(event_type)
        if not event_details:
            self._logger.error(f"No handler registered for event: {event_type}")
            return
        
        callback_func = event_details["callback"]
        if asyncio.iscoroutinefunction(callback_func):
            asyncio.create_task(callback_func(payload))
        else:
            callback_func(payload)
    
    # def add_handler(self, event_type, callback):
    #     self._handlers[event_type] = callback

    # def remove_handler(self, event_type, callback):
    #     self._handlers.pop(event_type)

    async def _start_websocket(self):
        self._logger.info("Starting websocket...")

        try:
            # Connect to the WebSocket server
            self._logger.info("Establishing connection to WebSocket server...")
            async with websockets.connect(self.WEBSOCKET_URL) as ws:
                self._logger.info("Connection succeeded. Starting up websocket handler...")

                # Handle incoming messages
                while True:
                    packet = await ws.recv()
                    # Process the message
                    self._logger.debug("\n\nReceived message: {}".format(packet))

                    message: dict = json.loads(packet)
                    metadata: dict = message.get("metadata", {})
                    message_type: str = metadata.get("message_type", None)

                    if message_type == "notification":
                        subscription_type = metadata["subscription_type"]
                        self._handle_event(subscription_type, message["payload"])
                        
                    # Handle session welcome message
                    elif message_type == "session_welcome":
                        self._logger.info("Received session welcome message")
                        
                        self._session_id = message["payload"]["session"]["id"]
                        # Subscribe to requested events
                        for event, val in self._handlers.items():
                            self._subscribe_to_event(event, val["version"])
                        
        except Exception as e:
            self._logger.error(f"Error connecting to WebSocket server: {e}")
            raise e
    
    def _subscribe_to_event(self, event_type, version):
        # https://dev.twitch.tv/docs/eventsub/manage-subscriptions/

        headers = {
            "Authorization": f"Bearer {self._access_token}",
            "Client-Id": self.client_id,
            "Content-Type": "application/json"  
        }
        body = {
            "type": event_type,
            "version": version,
            "condition": {
                "broadcaster_user_id": self.broadcaster_id,
                "user_id": "182465973",
                "moderator_user_id": self.moderator_user_id
            },
            "transport": {
                "method": "websocket",
                "session_id": self._session_id
            }
        }

        print(f"subscribing to {event_type}...")
        response = requests.post(self.API_URL + "/eventsub/subscriptions", headers=headers, json=body)

        # print response code
        # print(response.status_code)

        if response.status_code != 202:
            self._logger.error(f"Failed to subscribe to event {event_type}: {response.text}")
            raise Exception(response.text)
        else:
            self._logger.info(f"Subscribed to event {event_type}")