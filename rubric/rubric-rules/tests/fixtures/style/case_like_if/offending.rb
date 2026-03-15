if status == :active
  do_active
elsif status == :pending
  do_pending
elsif status == :cancelled
  do_cancelled
end

if type == "foo"
  handle_foo
elsif type == "bar"
  handle_bar
end
