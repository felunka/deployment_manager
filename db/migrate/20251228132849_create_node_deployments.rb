class CreateNodeDeployments < ActiveRecord::Migration[8.1]
  def change
    create_table :node_deployments do |t|
      t.references :node, null: false, foreign_key: true
      t.string :name, null: false
      t.string :path, null: false, default: "/home/node_agent/<NAME>"
      t.string :git_url, null: false
      t.integer :deployment_type, null: false, default: 0
      t.integer :deployment_status, null: false, default: 0
      t.string :compose

      t.timestamps
    end
  end
end
