db_url = ENV.fetch('DATABASE_URL')
secret = ENV.fetch('SECRET_KEY', nil)
port = ENV.fetch('PORT', '3000')
# ENV['DATABASE_URL'] is shown here only in a comment
msg = "use ENV['KEY'] is bad"
