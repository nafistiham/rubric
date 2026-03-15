begin
  do_something
rescue StandardError => e
  puts e.message
end

begin
  do_another
rescue => e
  puts e
end

begin
  risky
rescue TypeError, ArgumentError => e
  puts e
end
