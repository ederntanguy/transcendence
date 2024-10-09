from django.urls import path

from . import views

urlpatterns = [
    path("", views.base, name='home_page'),
    path("account", views.base, name='account'),
    path("chat", views.base, name='chat'),
    path("play", views.base, name='play'),
]
