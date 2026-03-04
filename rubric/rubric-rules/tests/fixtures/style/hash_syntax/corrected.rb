x = {a: 1, b: 2}

# Namespace separator :: is not a symbol rocket — must not be flagged
rescue ActiveRecord::RecordInvalid => e
  puts e

rescue ActiveRecord::RecordNotFound => e
  puts e

ERRORS = {
  ActiveRecord::RecordInvalid => 422,
  ActiveRecord::RecordNotFound => 404,
  HTTP::Error => 503,
}

# Mixed symbol and string keys — ruby19_no_mixed_keys leaves these alone
{ :out => @stdout_out, :err => @stderr_out, @stdout_in => :close }
post :create, params: { :domain_block => { domain: 'example.com' }, 'confirm' => '' }
image_tag(url, :class => 'foo', 'data-x' => 'bar')
