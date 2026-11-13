FROM nginx:1.27-alpine

RUN apk update && apk add --no-cache openssl curl

RUN mkdir -p /etc/nginx/ssl /usr/share/nginx/html

RUN rm -f /etc/nginx/conf.d/default.conf

COPY /tools/generate-ssl.sh /docker-entrypoint.d/10-generate-ssl.sh
RUN chmod +x /docker-entrypoint.d/10-generate-ssl.sh

COPY /conf/nginx.conf.template /etc/nginx/templates/transcendence.conf.template

EXPOSE ${NGINX_HTTPS_PORT}

CMD ["nginx", "-g", "daemon off;"]