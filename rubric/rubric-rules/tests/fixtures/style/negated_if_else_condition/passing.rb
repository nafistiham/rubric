# No else branch — not flagged
if !condition
  do_a
end

# Positive condition with else — not flagged
if condition
  do_a
else
  do_b
end

# unless with else is handled by UnlessElse cop
unless condition
  do_a
else
  do_b
end
