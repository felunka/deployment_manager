class User < ApplicationRecord
  has_secure_password
  encrypts :github_pat
end
