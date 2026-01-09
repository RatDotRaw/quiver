#!/usr/bin/env python

import time
import pika
credentials = pika.PlainCredentials('user', 'changeme')
parameters = pika.ConnectionParameters(host="localhost", port=5672, credentials=credentials)
connection = pika.BlockingConnection(parameters)
channel = connection.channel()


exchange_name = 'twitch_events'
queue_name = 'channel.chat.message'

# create queue and bind to exchange
# channel.queue_declare(queue=queue_name)
# channel.queue_bind(exchange=exchange_name, queue=queue_name)

count = 0

# run function on received message
def callback(ch, method, properties, body: bytes):
    global count
    print(f"[{count}] Received {body}")
    ch.basic_ack(delivery_tag = method.delivery_tag)
    count += 1

channel.basic_qos(prefetch_count=1) # set max messages of this worker
channel.basic_consume(queue=queue_name, on_message_callback=callback) # auto_ack=True => auto acknowledge message

channel.start_consuming()