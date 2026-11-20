FROM nginx:1.27-alpine@sha256:65645c7bb6a0661892a8b03b89d0743208a18dd2f3f17a54ef4b76fb8e2f2a10

RUN apk update && apk add --no-cache openssl curl

RUN mkdir -p /etc/nginx/ssl /usr/share/nginx/html

RUN rm -f /etc/nginx/conf.d/default.conf

COPY /tools/generate-ssl.sh /docker-entrypoint.d/10-generate-ssl.sh
RUN chmod +x /docker-entrypoint.d/10-generate-ssl.sh

COPY /conf/nginx.conf.template /etc/nginx/templates/transcendence.conf.template

EXPOSE ${NGINX_HTTPS_PORT}

CMD ["nginx", "-g", "daemon off;"]