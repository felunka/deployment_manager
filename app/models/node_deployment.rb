class NodeDeployment < ApplicationRecord
  belongs_to :node

  enum :deployment_type, {
    simple_docker_run: 0,
    simple_docker_compose: 1,
    github_action_runner: 2
  }
  enum :deployment_status, {
    pending_init: 0,
    init_failed: 1,
    healthy: 2,
    connection_lost: 3,
    decommissioned: 4
  }

  validates :name, presence: true
  validates :path, presence: true
end
