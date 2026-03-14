case foo
when 1
  puts "one"
when 2
  puts "two"
end

# Inline when with then is ok:
case foo
when 1 then puts "one"
end
