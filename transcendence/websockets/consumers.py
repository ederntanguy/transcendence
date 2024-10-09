import django
import os
import json
from operator import itemgetter

from channels.generic.websocket import AsyncWebsocketConsumer
from account.models import Relationship, RelationshipRequest, Player

os.environ.setdefault('DJANGO_SETTINGS_MODULE', 'transcendence.settings')
os.environ["DJANGO_ALLOW_ASYNC_UNSAFE"] = "true"
django.setup()


async def send_data(self, data):
    await self.send(text_data=data)


async def get_friends_list(player):
    all_relationship = []
    first_part = Relationship.objects.filter(first_user=player.id)
    second_part = Relationship.objects.filter(second_user=player.id)
    for relation in first_part:
        all_relationship.append(relation.second_user.username)
    for relation in second_part:
        all_relationship.append(relation.first_user.username)
    return all_relationship


def get_value_in_2d_array(array, value, index):
    for tmp in array:
        if tmp[index] == value:
            return tmp
    return False


async def update_all_info(player, self):
    friend_list = await get_friends_list(player)
    friend_list_with_status = []

    for friend in friend_list:
        if info := get_value_in_2d_array(self.friend_list_status, friend, 0):
            friend_list_with_status.append([info[0], info[1]])
        else:
            friend_list_with_status.append([friend, 'disconnected'])
    friend_list_with_status = sorted(friend_list_with_status, key=itemgetter(1))
    return json.dumps({
        "friends_list": friend_list_with_status,
        "friends_request_send": await get_friends_request_send(player),
        "friends_request_received": await get_friends_request_received(player)
    })


async def send_message_to_recipient(self, username, recipient, msg_type, info=None):
    message = {}
    if msg_type == 'update_relationship':
        message = {
            'type': 'update_friends_msg',
        }
    await self.channel_layer.group_send(
        recipient,
        message
    )


async def get_friends_request_send(player):
    all_friends_request_send = []
    relation_object = RelationshipRequest.objects.filter(request_user=player.id)
    for send in relation_object:
        all_friends_request_send.append(send.received_user.username)
    return all_friends_request_send


async def get_friends_request_received(player):
    all_friends_request_received = []
    relation_object = RelationshipRequest.objects.filter(received_user=player.id)
    for received in relation_object:
        all_friends_request_received.append(received.request_user.username)
    return all_friends_request_received


async def say_your_new_status_to_a_friend(friend, self, status, function_name):
    if function_name is not None:
        msg_type = function_name
    else:
        msg_type = 'update_friend_status'
    await self.channel_layer.group_send(
        friend,
        {
            'type': msg_type,
            'message': [self.username, status]
        }
    )


async def say_your_new_status_to_your_friends(self, status, function_name=None, new_username=None):
    if new_username is None:
        all_friends = await get_friends_list(Player.objects.filter(username=self.username)[0])
    else:
        all_friends = await get_friends_list(Player.objects.filter(username=new_username)[0])
    for friend in all_friends:
        await say_your_new_status_to_a_friend(friend, self, status, function_name)


async def request_new_friend(sender, receiver, self):
    if sender == receiver:
        return
    if RelationshipRequest.objects.filter(request_user=sender.id).filter(received_user=receiver.id).count() == 0:
        RelationshipRequest.objects.create(request_user=sender, received_user=receiver)
        await send_message_to_recipient(self, sender.username, receiver.username, 'update_relationship')


async def remove_friend(player, friend_player, self):
    if Relationship.objects.filter(first_user=player.id).filter(second_user=friend_player.id).count():
        Relationship.objects.filter(first_user=player.id).filter(second_user=friend_player.id).delete()
    else:
        Relationship.objects.filter(first_user=friend_player.id).filter(second_user=player.id).delete()
    await say_your_new_status_to_a_friend(friend_player.username, self, 'disconnected', None)
    await send_message_to_recipient(self, player.username, friend_player.username, 'update_relationship')


async def cancel_friend_request(player, friend_player, self):
    RelationshipRequest.objects.filter(request_user=player.id).filter(received_user=friend_player.id).delete()
    await send_message_to_recipient(self, player.username, friend_player.username, 'update_relationship')


async def accept_friend_request(player, friend_player, self):
    RelationshipRequest.objects.filter(request_user=friend_player.id).filter(received_user=player.id).delete()
    Relationship.objects.create(first_user=friend_player, second_user=player)
    await say_your_new_status_to_a_friend(friend_player.username, self, None, 'check_is_friend_connected')
    await send_message_to_recipient(self, player.username, friend_player.username, 'update_relationship')


async def refuse_friend_request(player, friend_player, self):
    RelationshipRequest.objects.filter(request_user=friend_player.id).filter(received_user=player.id).delete()
    await send_message_to_recipient(self, player.username, friend_player.username, 'update_relationship')


async def update_each_part_of_relation(username, self, data):
    for user in data:
        await send_message_to_recipient(self, username, user, 'update_relationship')


async def update_username_to_all_relation(username, self):
    player = Player.objects.filter(username=username)[0]
    await update_each_part_of_relation(username, self, await get_friends_list(player))
    await update_each_part_of_relation(username, self, await get_friends_request_received(player))
    await update_each_part_of_relation(username, self, await get_friends_request_send(player))


async def delete_all_friends(player, self):
    for friend in await get_friends_list(player):
        await remove_friend(player, Player.objects.filter(username=friend)[0], self)
    for received in await get_friends_request_received(player):
        await remove_friend(player, Player.objects.filter(username=received)[0], self)
    for send in await get_friends_request_send(player):
        await remove_friend(player, Player.objects.filter(username=send)[0], self)


async def update_username(self, new_username):
    await say_your_new_status_to_your_friends(self, 'disconnected', None, new_username)
    await self.channel_layer.group_discard(
        self.username,
        self.channel_name
    )
    self.username = username = new_username
    await self.channel_layer.group_add(
        self.username,
        self.channel_name
    )
    await say_your_new_status_to_your_friends(self, 'connected')
    await update_username_to_all_relation(username, self)
    return self, Player.objects.filter(username=username)[0]


def update_status_of_a_friend(self, username, new_status):
    for i in range(len(self.friend_list_status)):
        if self.friend_list_status[i][0] == username:
            self.friend_list_status[i] = [username, new_status]
            return
    self.friend_list_status.append([username, new_status])


async def handle_type_received(player, friend_player, data, self):
    if data[0] == 'request_new_friend':
        await request_new_friend(player, friend_player, self)
    elif data[0] == 'remove_friend':
        await remove_friend(player, friend_player, self)
    elif data[0] == 'cancel_friend_request':
        await cancel_friend_request(player, friend_player, self)
    elif data[0] == 'accept_friend_request':
        await accept_friend_request(player, friend_player, self)
    elif data[0] == 'refuse_friend_request':
        await refuse_friend_request(player, friend_player, self)
    elif data[0] == 'update_username':
        self, player = await update_username(self, data[1])
    elif data[0] == 'delete_user':
        await delete_all_friends(player, self)
        return -1, -1
    elif data[0] == 'in game':
        await say_your_new_status_to_your_friends(self, 'in game')
    elif data[0] == 'game done':
        await say_your_new_status_to_your_friends(self, 'connected')
    return self, player


class PlayerCommunications(AsyncWebsocketConsumer):
    async def connect(self):
        # Join room group
        self.username = self.scope['url_route']['kwargs']['username']
        if Player.objects.filter(username=self.username).count() == 0:
            self.username = ''
            return
        self.friend_list_status = []
        await self.channel_layer.group_add(
            self.username,
            self.channel_name
        )
        await say_your_new_status_to_your_friends(self, None, 'check_is_friend_connected')
        await self.accept()

    async def disconnect(self, close_code):
        # Leave room group
        if Player.objects.filter(username=self.username).count() == 0:
            return
        await say_your_new_status_to_your_friends(self, 'disconnected')
        await self.channel_layer.group_discard(
            self.username,
            self.channel_name
        )
        pass

    # Receive message from WebSocket
    async def receive(self, text_data):
        data = json.loads(text_data)
        username = self.username
        player = None

        friend_player = None
        if data[0] != 'update_username':
            if Player.objects.filter(username=self.username).count() == 0:
                return
            player = Player.objects.filter(username=username)[0]
            if len(data) > 1:
                if Player.objects.filter(username=data[1]).count() == 1:
                    friend_player = Player.objects.filter(username=data[1])[0]
                else:
                    await send_data(self, await update_all_info(player, self))
                    return

        self, player = await handle_type_received(player, friend_player, data, self)
        if self != -1 and player != -1:
            await send_data(self, await update_all_info(player, self))

    # Receive message from room group
    async def update_friends_msg(self, event):
        await send_data(self, await update_all_info(Player.objects.filter(username=self.username)[0], self))

    async def check_is_friend_connected(self, event):
        username = event['message'][0]
        update_status_of_a_friend(self, username, 'connected')
        message = {
            'type': 'update_friend_status',
            'message': [self.username, 'connected']
        }
        await self.channel_layer.group_send(
            username,
            message
        )
        await send_data(self, await update_all_info(Player.objects.filter(username=self.username)[0], self))

    async def update_friend_status(self, event):
        info = event['message']
        update_status_of_a_friend(self, info[0], info[1])
        await send_data(self, await update_all_info(Player.objects.filter(username=self.username)[0], self))
