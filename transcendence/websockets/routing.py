from django.urls import path

from . import consumers

websocket_urlpatterns = [
    path("ws/friends/<username>/", consumers.PlayerCommunications.as_asgi()),
]
