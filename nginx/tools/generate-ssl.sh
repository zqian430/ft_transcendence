#!/bin/sh
set -e

SSL_DIR="/etc/nginx/ssl"
CERT_FILE="$SSL_DIR/transcendence.crt"
KEY_FILE="$SSL_DIR/transcendence.key"

# Read environment variables with fallback values
DOMAIN=${DOMAIN_NAME:-localhost}
C=${SSL_COUNTRY:-FR}
ST=${SSL_STATE:-IDF}
L=${SSL_CITY:-Lyon}
O=${SSL_ORGANIZATION:-42}
OU=${SSL_ORG_UNIT:-ft_transcendence}

if [ ! -f "$CERT_FILE" ]; then
    echo "-> Génération du certificat SSL auto-signé pour : $DOMAIN"
    openssl req -x509 -nodes -days 365 -newkey rsa:2048 \
        -keyout "$KEY_FILE" \
        -out "$CERT_FILE" \
        -subj "/C=$C/ST=$ST/L=$L/O=$O/OU=$OU/CN=$DOMAIN"
    chmod 600 "$KEY_FILE"
    chmod 644 "$CERT_FILE"
    echo "-> Certificat et clé générés dans $SSL_DIR."
else
    echo "-> Certificat SSL existant détecté, étape ignorée."
fi