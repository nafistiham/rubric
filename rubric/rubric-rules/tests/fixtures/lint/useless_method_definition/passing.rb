def foo
  super
  do_extra_work
end

def bar
  modified = transform(super)
  modified
end
