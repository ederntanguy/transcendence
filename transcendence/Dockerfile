FROM python:3.11.7-slim-bookworm


RUN apt update; apt -y upgrade; apt -y install libpq5 curl postgresql postgresql-client; apt -y autoremove ; \
    mkdir -p /home/website;

COPY requirements.txt ./
RUN pip install --upgrade pip; \
    pip install --no-cache-dir -r requirements.txt; \
    rm ./requirements.txt; \
    find /home/website/ -type d \( -name '__pycache__' -o -name 'migrations' \) -prune -exec rm -rf {} \;


COPY .pg_service.conf /root/


COPY run_django.sh /usr/bin/


COPY account/ /home/website/account/
COPY websockets/ /home/website/websockets/
COPY navigation/ /home/website/navigation/
COPY static/ /home/website/static/
COPY --chmod=0666 ssl/transcendence.crt /home/website/ssl/
COPY --chmod=0400 ssl/transcendence.key /home/website/ssl/
COPY templates/ /home/website/templates/
COPY transcendence/ /home/website/transcendence/
COPY manage.py /home/website/

WORKDIR /home/website/


EXPOSE 43443
EXPOSE 6379

ENTRYPOINT ["bash", "/usr/bin/run_django.sh"]
