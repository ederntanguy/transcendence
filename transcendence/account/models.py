from django.db import models


class Player(models.Model):
    username = models.CharField(max_length=64)
    password = models.BinaryField()
    tournament_username = models.CharField(max_length=64, default='')

    def __str__(self):
        return self.username


class GameResult(models.Model):
    p1 = models.ForeignKey(Player, on_delete=models.SET_DEFAULT, default=1, related_name="p1")
    p2 = models.ForeignKey(Player, on_delete=models.SET_DEFAULT, default=1, related_name="p2")
    p1_score = models.PositiveSmallIntegerField(default=0)
    p2_score = models.PositiveSmallIntegerField(default=0)
    winner = models.ForeignKey(Player, on_delete=models.SET_DEFAULT, default=1, related_name="winner")
    date = models.DateTimeField()
    duration = models.DurationField()


class Relationship(models.Model):
    first_user = models.ForeignKey(Player, on_delete=models.CASCADE, default=1, related_name="first_user")
    second_user = models.ForeignKey(Player, on_delete=models.CASCADE, related_name="second_user")


class RelationshipRequest(models.Model):
    request_user = models.ForeignKey(Player, on_delete=models.CASCADE, default=1, related_name="request_user")
    received_user = models.ForeignKey(Player, on_delete=models.CASCADE, related_name="received_user")
