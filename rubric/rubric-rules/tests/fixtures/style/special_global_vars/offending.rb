# These should be flagged
puts $0
$: << '/usr/local/lib'
rescue => e
  puts $!
end

if $@ && !$@.empty?
  warn $@.first
end

sep = $;
out = $>
