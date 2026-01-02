class ChangeColumnDefaultForNodePort < ActiveRecord::Migration[8.1]
  def change
    change_column_default :nodes, :port, 443
  end
end
