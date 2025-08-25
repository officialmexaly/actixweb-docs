#!/bin/bash

# Complete PostgreSQL setup script for tech_docs
# This script will set up everything from scratch

set -e  # Exit on any error

echo "🚀 Setting up PostgreSQL for tech_docs backend..."
echo ""

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Function to print colored output
print_status() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Check if PostgreSQL is installed and running
print_status "Checking PostgreSQL installation..."
if ! command -v psql &> /dev/null; then
    print_error "PostgreSQL is not installed!"
    echo "Install it with: sudo apt install postgresql postgresql-contrib"
    exit 1
fi

if ! sudo systemctl is-active --quiet postgresql; then
    print_warning "PostgreSQL is not running. Starting it..."
    sudo systemctl start postgresql
    sleep 2
fi

print_success "PostgreSQL is running!"

# Get password for the new user
echo ""
print_status "Setting up database user and database..."
read -p "Enter password for tech_docs_user (or press Enter for 'securepass123'): " USER_PASSWORD
USER_PASSWORD=${USER_PASSWORD:-securepass123}

# Set up database and user
print_status "Creating database and user..."
sudo -u postgres psql << EOF
-- Drop existing database and user if they exist
DROP DATABASE IF EXISTS tech_docs;
DROP USER IF EXISTS tech_docs_user;

-- Create user with password
CREATE USER tech_docs_user WITH ENCRYPTED PASSWORD '$USER_PASSWORD';

-- Create database owned by the user
CREATE DATABASE tech_docs OWNER tech_docs_user;

-- Connect to the database and set permissions
\c tech_docs

-- Grant all permissions on public schema
GRANT ALL ON SCHEMA public TO tech_docs_user;
ALTER SCHEMA public OWNER TO tech_docs_user;

-- Grant default permissions for future objects
ALTER DEFAULT PRIVILEGES IN SCHEMA public GRANT ALL ON TABLES TO tech_docs_user;
ALTER DEFAULT PRIVILEGES IN SCHEMA public GRANT ALL ON SEQUENCES TO tech_docs_user;

-- Create UUID extension (needed for UUID columns)
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

EOF

if [ $? -eq 0 ]; then
    print_success "Database and user created successfully!"
else
    print_error "Failed to create database and user!"
    exit 1
fi

# Test the connection
print_status "Testing database connection..."
if PGPASSWORD=$USER_PASSWORD psql -U tech_docs_user -d tech_docs -h localhost -c "SELECT 1;" > /dev/null 2>&1; then
    print_success "Database connection test successful!"
else
    print_warning "Direct connection test failed. This might be due to authentication settings."
    print_status "Checking if we need to fix PostgreSQL authentication..."
    
    # Find pg_hba.conf
    PG_HBA_CONF=$(sudo -u postgres psql -t -c "SHOW hba_file;" | xargs)
    print_status "Found pg_hba.conf at: $PG_HBA_CONF"
    
    # Check current auth settings
    if sudo grep -q "local.*all.*all.*peer" "$PG_HBA_CONF"; then
        print_warning "Found 'peer' authentication. Need to change to 'md5'."
        
        # Ask user permission
        read -p "Do you want to change PostgreSQL authentication to allow password login? (y/N): " -n 1 -r
        echo
        if [[ $REPLY =~ ^[Yy]$ ]]; then
            # Backup config
            sudo cp "$PG_HBA_CONF" "$PG_HBA_CONF.backup.$(date +%Y%m%d_%H%M%S)"
            
            # Modify authentication
            sudo sed -i \
                -e 's/^local\s\+all\s\+all\s\+peer$/local   all             all                                     md5/' \
                -e '/^host.*all.*all.*127\.0\.0\.1\/32.*ident$/c\host    all             all             127.0.0.1/32            md5' \
                -e '/^host.*all.*all.*::1\/128.*ident$/c\host    all             all             ::1/128                 md5' \
                "$PG_HBA_CONF"
            
            # Restart PostgreSQL
            print_status "Restarting PostgreSQL..."
            sudo systemctl restart postgresql
            sleep 2
            
            # Test again
            if PGPASSWORD=$USER_PASSWORD psql -U tech_docs_user -d tech_docs -h localhost -c "SELECT 1;" > /dev/null 2>&1; then
                print_success "Database connection now works!"
            else
                print_error "Connection still fails. Please check the manual setup guide."
            fi
        fi
    fi
fi

# Create .env file
print_status "Creating .env file..."
cat > .env << EOF
DATABASE_URL=postgres://tech_docs_user:$USER_PASSWORD@localhost:5432/tech_docs
SERVER_HOST=127.0.0.1
SERVER_PORT=8080
RUST_LOG=info
EOF

print_success ".env file created!"

# Final test with the actual connection string
print_status "Testing connection with the exact connection string from .env..."
if PGPASSWORD=$USER_PASSWORD psql "postgres://tech_docs_user:$USER_PASSWORD@localhost:5432/tech_docs" -c "SELECT 'Connection successful!' as status;" 2>/dev/null; then
    print_success "Perfect! The connection string works!"
else
    print_warning "Connection string test failed, but this might still work with the Rust app."
fi

echo ""
print_success "Setup complete! Your configuration:"
echo "  Database: tech_docs"
echo "  User: tech_docs_user"
echo "  Password: $USER_PASSWORD"
echo "  Connection: postgres://tech_docs_user:$USER_PASSWORD@localhost:5432/tech_docs"
echo ""
print_status "Now you can run your Rust application:"
echo "  cargo run"
echo ""
print_status "If you still get connection errors, try:"
echo "  1. Check if .env file is in the project root"
echo "  2. Verify PostgreSQL is running: sudo systemctl status postgresql"
echo "  3. Test connection manually: psql 'postgres://tech_docs_user:$USER_PASSWORD@localhost:5432/tech_docs'"