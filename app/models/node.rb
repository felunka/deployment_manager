class Node < ApplicationRecord
  enum :node_status, {
    pending_init: 0,
    init_failed: 1,
    healthy: 2,
    connection_lost: 3,
    decommissioned: 4
  }
  encrypts :key

  has_many :node_deployments

  validates :hostname, presence: true
  validates :ip, presence: true, format: {
    with: /\A(
      (25[0-5]|2[0-4][0-9]|1[0-9]{2}|[1-9]?[0-9])\.  # IPv4 octet 1
      (25[0-5]|2[0-4][0-9]|1[0-9]{2}|[1-9]?[0-9])\.  # IPv4 octet 2
      (25[0-5]|2[0-4][0-9]|1[0-9]{2}|[1-9]?[0-9])\.  # IPv4 octet 3
      (25[0-5]|2[0-4][0-9]|1[0-9]{2}|[1-9]?[0-9])    # IPv4 octet 4
    |
      (
        ([0-9A-Fa-f]{1,4}:){7}[0-9A-Fa-f]{1,4}|
        ([0-9A-Fa-f]{1,4}:){1,7}:|
        :(:[0-9A-Fa-f]{1,4}){1,7}|
        ([0-9A-Fa-f]{1,4}:){1,6}:[0-9A-Fa-f]{1,4}|
        ([0-9A-Fa-f]{1,4}:){1,5}(:[0-9A-Fa-f]{1,4}){1,2}|
        ([0-9A-Fa-f]{1,4}:){1,4}(:[0-9A-Fa-f]{1,4}){1,3}|
        ([0-9A-Fa-f]{1,4}:){1,3}(:[0-9A-Fa-f]{1,4}){1,4}|
        ([0-9A-Fa-f]{1,4}:){1,2}(:[0-9A-Fa-f]{1,4}){1,5}|
        [0-9A-Fa-f]{1,4}:(:[0-9A-Fa-f]{1,4}){1,6}|
        :(:[0-9A-Fa-f]{1,4}){1,6}
      )
    )\z/x
  }
  validates :api_url, presence: true
  validates :port, presence: true
  validates :key, presence: true, length: { minimum: 16 }

  def to_s
    "#{self.hostname} (#{self.ip})"
  end
end
