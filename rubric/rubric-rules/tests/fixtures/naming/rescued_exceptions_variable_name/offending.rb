begin
  do_something
rescue StandardError => exception
  puts exception.message
end

begin
  do_another
rescue => error
  puts error
end

begin
  risky
rescue TypeError => ex
  puts ex
end
