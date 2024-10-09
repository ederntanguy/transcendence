import json
import os
import urllib.parse
import shutil
from django.shortcuts import render
from django.http import HttpResponse
from django.conf import settings
from argon2 import PasswordHasher
from django.core.files.storage import FileSystemStorage
from navigation.views import remove_unknown_user_cookies
from account.models import Player, GameResult

ph = PasswordHasher()


def create_pfp(path, credentials):
    if not credentials.FILES.get('pfp', False):
        return
    profile_picture = credentials.FILES['pfp']
    name, ext = os.path.splitext(profile_picture.name)
    if ext == '.jpg':
        ext = '.jpeg'
    if os.path.isfile(f'{path}pfp.png'):
        os.remove(f'{path}pfp.png')
    elif os.path.isfile(f'{path}pfp.jpeg'):
        os.remove(f'{path}pfp.jpeg')
    FileSystemStorage(location=path).save(f'pfp{ext}', profile_picture)


def update_username(path, player, new_username):
    if new_username != '' and new_username != player.username:
        if Player.objects.filter(username=new_username).count() != 0:
            return -1, -1
        player.username = new_username
        new_path = os.path.join(settings.MEDIA_ROOT, f'account/{new_username}/')
        os.rename(path, new_path)
        path = new_path
        player.save()
    return player, path


def update_password(player, credentials):
    new_password = credentials.POST.get('np')
    if new_password != '':
        player.password = ph.hash(new_password).encode('utf-8')
    return player


def make_success_response(player):
    response = HttpResponse("0")
    response.set_cookie(key="username", value=urllib.parse.quote(player.username, safe=""), secure=True, samesite='Lax')
    if os.path.isfile(os.path.join(settings.MEDIA_ROOT, f'account/{player.username}/pfp.png')):
        pfp_uri = f'/media/account/{player.username}/pfp.png'
    elif os.path.isfile(os.path.join(settings.MEDIA_ROOT, f'account/{player.username}/pfp.jpeg')):
        pfp_uri = f'/media/account/{player.username}/pfp.jpeg'
    else:
        pfp_uri = '/static/account/default.png'
    response.set_cookie(key="pfp_uri", value=urllib.parse.quote(pfp_uri, safe=""), secure=True, samesite='Lax')
    if player.tournament_username != '':
        response.set_cookie(key="tournament_username", value=urllib.parse.quote(player.tournament_username, safe=""), secure=True, samesite='Lax')
    else:
        response.delete_cookie('tournament_username', samesite='Lax')
    return response


def valid_username(username, username_type):
    if len(username) >= 64:
        return f"1{username_type} too long. Please remove characters."
    for char in username:
        if not char.isalpha() and not (char == '.' or char == '-' or char == '_'):
            return f"1Your {username_type} need to contain only alphanumeric, '-', '_', or '.'"
    return ''


def signup_handler(credentials):
    username = credentials.POST.get('u')
    if (error := valid_username(username, "username")) != '':
        return HttpResponse(error)
    if Player.objects.filter(username=username).count() == 0:
        hashed_password = ph.hash(credentials.POST.get('p')).encode('utf-8')
        path = os.path.join(settings.MEDIA_ROOT, f'account/{username}/')
        try:
            os.mkdir(path)
        except:
            return HttpResponse("1Username contains invalid characters. Try removing slashes.")
        create_pfp(path, credentials)
        Player.objects.create(username=username, password=hashed_password)
        return make_success_response(Player.objects.get(username=username))
    else:
        return HttpResponse("1Username is taken.")


def signin_handler(player, _):
    return make_success_response(player)


def update_tournament_username(player, new_tournament_username):
    error = valid_username(new_tournament_username, "tournament username")
    if error != '':
        return error
    if Player.objects.filter(tournament_username=new_tournament_username).count() != 0:
        return "1The tournament username is already used."
    if new_tournament_username != '':
        player.tournament_username = new_tournament_username
    return player


def update_handler(player, credentials):
    path = os.path.join(settings.MEDIA_ROOT, f'account/{player.username}/')
    try:
        new_username = credentials.POST.get('nu')
        if (error := valid_username(new_username, "username")) != '':
            return HttpResponse(error)
        player, path = update_username(path, player, new_username)
        if player == -1 and path == -1:
            return HttpResponse("1The username is already used.")
    except:
        return HttpResponse("1Username contains invalid characters. Try removing slashes.")
    create_pfp(path, credentials)
    player = update_password(player, credentials)

    new_tournament_username = credentials.POST.get('ntu')
    if new_tournament_username != '' and new_tournament_username != player.tournament_username:
        player = update_tournament_username(player, new_tournament_username)
        if isinstance(player, str):
            return HttpResponse(player)

    player.save()
    return make_success_response(player)


def check_auth(credentials, and_then):
    username = credentials.POST.get('u')
    try:
        player = Player.objects.get(username=username)
        try:
            ph.verify(player.password, credentials.POST.get('p'))
            return and_then(player, credentials)
        except:
            return HttpResponse("1Invalid password")
    except:
        return HttpResponse("1Invalid username.")


def check_credentials(request, and_then):
    if request.method != 'POST':
        return remove_unknown_user_cookies(request, render(request, 'base.html'))
    if request.POST.get('u') == "" or request.POST.get('p') == "":
        return HttpResponse("1Please provide both username and password.")
    return and_then(request)


def user_signup(request):
    return check_credentials(request, signup_handler)


def user_signin(request):
    return check_credentials(request, lambda r: check_auth(r, signin_handler))


def user_update(request):
    return check_credentials(request, lambda r: check_auth(r, update_handler))


def user_logout(request):
    if request.method != 'POST':
        response = render(request, 'base.html')
    else:
        response = HttpResponse("0")
    response.delete_cookie('username', samesite='Lax')
    response.delete_cookie('pfp_uri', samesite='Lax')
    response.delete_cookie('tournament_username', samesite='Lax')
    return response


def user_delete(request):
    username = request.COOKIES.get('username')
    if Player.objects.filter(username=username).count():
        Player.objects.filter(username=username).delete()
        shutil.rmtree(os.path.join(settings.MEDIA_ROOT, f'account/{username}/'))
    return user_logout(request)


def result_info_in_json(result, pos_own_id):
    if pos_own_id == 1:
        own_id = result.p1_id
        opponent_id = result.p2_id
        own_score = result.p1_score
        opponent_score = result.p2_score
    else:
        own_id = result.p2_id
        opponent_id = result.p1_id
        own_score = result.p2_score
        opponent_score = result.p1_score

    tmp = {
        "own_username": Player.objects.filter(id=own_id)[0].username,
        "opponent_username": Player.objects.filter(id=opponent_id)[0].username,
        "own_score": own_score,
        "opponent_score": opponent_score,
        "winner": Player.objects.filter(id=result.winner_id)[0].username,
        "date": result.date.strftime("%m/%d/%Y, %H:%M:%S"),
        "duration": str(result.duration),
    }
    return tmp


def get_user_statistics(request):
    username = request.COOKIES.get('username')
    if Player.objects.filter(username=username).count() == 0:
        return HttpResponse('1Error username not found.')
    result_first_part = GameResult.objects.filter(p1_id=Player.objects.filter(username=username)[0].id)
    result_second_part = GameResult.objects.filter(p2_id=Player.objects.filter(username=username)[0].id)

    all_result = []

    for result in result_first_part:
        all_result.insert(0, result_info_in_json(result, 1))

    # sort by the newest game
    for result in result_second_part:
        info = result_info_in_json(result, 2)
        for i in range(len(all_result)):
            if all_result[i].get('date') == info.get('date'):
                break
            if all_result[i].get('date') < info.get('date'):
                all_result.insert(i, info)
                break
            if i + 1 == len(all_result):
                all_result.append(info)
        if len(all_result) == 0:
            all_result.append(info)
    return HttpResponse(json.dumps(all_result))
