# These should not be flagged
if foo && bar
  do_something
end

if foo || bar
  do_something
end

# Bitwise AND with mask is arithmetic, not a boolean — but on if-condition
# is still valid Ruby. We only skip &&/|| and &./&=/|= forms.
x = a & 0xFF
y = b | 0x0F

if object&.method
  call_safe
end
