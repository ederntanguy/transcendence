from django.shortcuts import render
from account.models import Player
from django.http import HttpResponse
from account.models import Relationship
import json


def remove_unknown_user_cookies(request, response):
    if request.COOKIES.get('username', False):
        if Player.objects.filter(username=request.COOKIES.get('username')).count() == 0:
            response.delete_cookie('pfp_uri')
            response.delete_cookie('username')
    return response


def base(request):
    return remove_unknown_user_cookies(request, render(request, 'base.html'))
