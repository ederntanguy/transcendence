#!/bin/sh

openssl genpkey -out root_ca.pem.key -outform PEM -algorithm RSA -pkeyopt rsa_keygen_bits:4096
openssl pkcs8 -topk8 -inform PEM -outform DER -in ./root_ca.pem.key -out root_ca.der.key -nocrypt
openssl req -outform PEM -out root_ca.pem.crt -new -key ./root_ca.pem.key -keyform PEM -x509 -days 90 <<EOF
KK
Fartland
Pisstown
Farting Inc
The Cumzone
Biggus Dickus
biggus.dickus@cumzone.farting.kk
EOF
openssl x509 -inform PEM -outform DER -in ./root_ca.pem.crt -out root_ca.der.crt
openssl genpkey -out transcendence.pem.key -outform PEM -algorithm RSA -pkeyopt rsa_keygen_bits:4096
openssl pkcs8 -topk8 -inform PEM -outform DER -in ./transcendence.pem.key -out transcendence.der.key -nocrypt
openssl req -outform PEM -out transcendence.pem.csr -new -key transcendence.pem.key -keyform PEM -addext subjectAltName=DNS:localhost <<EOF
FR
Ile-de-France
Paris
42
.
transcendence
transcendence@42.fr
abcde
.
EOF
openssl req -inform PEM -outform PEM -in ./transcendence.pem.csr -out transcendence.pem.crt -x509 -CA root_ca.pem.crt -CAkey root_ca.pem.key -days 90 -copy_extensions none
openssl x509 -inform PEM -outform DER -in ./transcendence.pem.crt -out transcendence.der.crt

mkdir -p ../transcendence/ssl
cp ./transcendence.pem.key ../transcendence/ssl/transcendence.key
cp ./transcendence.pem.crt ../transcendence/ssl/transcendence.crt
mkdir -p ../pong-serv/tls
cp ./transcendence.der.key ../pong-serv/tls/transcendence.der.key
cp ./transcendence.der.crt ../pong-serv/tls/transcendence.der.crt
cp ./root_ca.pem.crt ../pong-serv/tls/root_ca.pem.crt
mkdir -p ../nginx/tls
cp ./transcendence.pem.key ../nginx/tls/transcendence.pem.key
cp ./transcendence.pem.crt ../nginx/tls/transcendence.pem.crt
