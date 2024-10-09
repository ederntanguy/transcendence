from django.urls import path

from . import views

urlpatterns = [
    path("account/signup", views.user_signup, name='signup'),
    path("account/signin", views.user_signin, name='signin'),
    path("account/logout", views.user_logout, name='logout'),
    path("account/delete", views.user_delete, name='delete'),
    path("account/statistics", views.get_user_statistics, name='statistics'),
    path("account/update", views.user_update, name='update'),
]
