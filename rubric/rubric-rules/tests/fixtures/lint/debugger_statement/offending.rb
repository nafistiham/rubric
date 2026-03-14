def foo
  binding.pry
  byebug
  bar
end

def baz
  debugger
  remote_byebug
end
