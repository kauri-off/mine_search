#!/bin/bash

# Request domain
read -p "Enter your domain: " DOMAIN

# Input validation
if [[ -z "$DOMAIN" ]]; then
    echo "Error: Domain not entered!"
    exit 1
fi

# Select service
read -p "Which service to use? (nginx/caddy/both): " SERVICE

# File paths
NGINX_TEMPLATE="nginx.conf"
CADDY_TEMPLATE="Caddyfile"
NGINX_DEST="/etc/nginx/sites-enabled/default"
CADDY_DEST="/etc/caddy/Caddyfile"

# Update configuration based on selected service
if [[ "$SERVICE" == "nginx" || "$SERVICE" == "both" ]]; then
    sed "s/yourdomain.com/$DOMAIN/g" "$NGINX_TEMPLATE" > "$NGINX_DEST"
    systemctl restart nginx
    echo "Nginx updated and restarted."
fi

if [[ "$SERVICE" == "caddy" || "$SERVICE" == "both" ]]; then
    sed "s/yourdomain.com/$DOMAIN/g" "$CADDY_TEMPLATE" > "$CADDY_DEST"
    systemctl restart caddy
    echo "Caddy updated and restarted."
fi

if [[ "$SERVICE" != "nginx" && "$SERVICE" != "caddy" && "$SERVICE" != "both" ]]; then
    echo "Error: Invalid service selection."
    exit 1
fi

echo "Configuration updated."