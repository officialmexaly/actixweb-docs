#!/bin/bash

# Fix PostgreSQL authentication issues
# This script will configure PostgreSQL to use password authentication

echo "🔧 Fixing PostgreSQL authentication..."

# Find PostgreSQL version and config file location
PG_VERSION=$(sudo -u postgres psql -t -c "SELECT version();" | grep -oP '\d+\.\d+' | head -1)
echo "📍 PostgreSQL version: $PG_VERSION"

# Common config file locations
CONFIG_LOCATIONS=(
    "/etc/postgresql/$PG_VERSION/main/pg_hba.conf"
    "/etc/postgresql/*/main/pg_hba.conf"
    "/usr/local/etc/postgresql/pg_hba.conf"
    "/var/lib/pgsql/data/pg_hba.conf"
)

PG_HBA_CONF=""
for location in "${CONFIG_LOCATIONS[@]}"; do
    if [ -f "$location" ] || ls $location 1> /dev/null 2>&1; then
        PG_HBA_CONF=$(ls $location 2>/dev/null | head -1)
        break
    fi
done

if [ -z "$PG_HBA_CONF" ]; then
    echo "❌ Could not find pg_hba.conf file. Please locate it manually."
    echo "   Try: sudo find /etc -name 'pg_hba.conf' 2>/dev/null"
    exit 1
fi

echo "📁 Found config file: $PG_HBA_CONF"

# Backup original config
echo "💾 Creating backup..."
sudo cp "$PG_HBA_CONF" "$PG_HBA_CONF.backup.$(date +%Y%m%d_%H%M%S)"

# Show current config
echo "📋 Current authentication config:"
sudo grep -E "^(local|host).*all.*all" "$PG_HBA_CONF" || echo "No matching lines found"

echo ""
echo "🔄 This script will modify PostgreSQL authentication to allow password login."
echo "   This will change 'peer' authentication to 'md5' (password) authentication."
echo ""
read -p "❓ Continue? (y/N): " -n 1 -r
echo
if [[ ! $REPLY =~ ^[Yy]$ ]]; then
    echo "❌ Aborted."
    exit 1
fi

# Modify pg_hba.conf to use md5 authentication
echo "✏️  Updating authentication methods..."
sudo sed -i.bak \
    -e 's/^local\s\+all\s\+all\s\+peer$/local   all             all                                     md5/' \
    -e 's/^local\s\+all\s\+postgres\s\+peer$/local   all             postgres                                peer/' \
    -e '/^host.*all.*all.*127\.0\.0\.1\/32.*ident$/c\host    all             all             127.0.0.1/32            md5' \
    -e '/^host.*all.*all.*::1\/128.*ident$/c\host    all             all             ::1/128                 md5' \
    "$PG_HBA_CONF"

# Show new config
echo "📋 New authentication config:"
sudo grep -E "^(local|host).*all.*all" "$PG_HBA_CONF"

# Restart PostgreSQL
echo "🔄 Restarting PostgreSQL..."
sudo systemctl restart postgresql

# Wait a moment for PostgreSQL to start
sleep 2

# Check PostgreSQL status
if sudo systemctl is-active --quiet postgresql; then
    echo "✅ PostgreSQL restarted successfully!"
else
    echo "❌ PostgreSQL failed to restart. Checking status..."
    sudo systemctl status postgresql
    echo ""
    echo "🔄 Restoring backup config..."
    sudo cp "$PG_HBA_CONF.backup."* "$PG_HBA_CONF"
    sudo systemctl restart postgresql
    echo "❌ Authentication fix failed. Original config restored."
    exit 1
fi

echo ""
echo "🎉 Authentication fix complete!"
echo ""
echo "📝 Now set up the database and user:"
echo "   1. Create user with password:"
echo "      sudo -u postgres psql -c \"CREATE USER tech_docs_user WITH ENCRYPTED PASSWORD 'your_secure_password';\""
echo ""
echo "   2. Create database:"
echo "      sudo -u postgres psql -c \"CREATE DATABASE tech_docs OWNER tech_docs_user;\""
echo ""
echo "   3. Test connection:"
echo "      psql -U tech_docs_user -d tech_docs -h localhost -W"
echo ""
echo "   4. Update your .env file:"
echo "      DATABASE_URL=postgres://tech_docs_user:your_secure_password@localhost:5432/tech_docs"
