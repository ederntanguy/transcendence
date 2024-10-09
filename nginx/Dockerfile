FROM nginx:1.25.3-alpine

COPY nginx.conf /etc/nginx/nginx.conf
COPY --chown=nginx:nginx --chmod=0666 tls/transcendence.pem.crt /etc/nginx/tls/
COPY --chown=nginx:nginx --chmod=0400 tls/transcendence.pem.key /etc/nginx/tls/

EXPOSE 8080
