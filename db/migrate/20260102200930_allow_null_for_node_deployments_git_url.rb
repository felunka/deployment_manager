class AllowNullForNodeDeploymentsGitUrl < ActiveRecord::Migration[8.1]
  def change
    change_column_null :node_deployments, :git_url, true
  end
end
