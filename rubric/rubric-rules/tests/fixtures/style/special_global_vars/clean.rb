# These should not be flagged
puts $PROGRAM_NAME
$LOAD_PATH << '/usr/local/lib'
rescue => e
  puts $ERROR_INFO
end

if $ERROR_POSITION && !$ERROR_POSITION.empty?
  warn $ERROR_POSITION.first
end

sep = $FIELD_SEPARATOR
out = $DEFAULT_OUTPUT

# $0 in a comment is fine
# $! in a comment is fine
