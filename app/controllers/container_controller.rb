class ContainerController < ApplicationController
  def index
    @node = Node.find params[:node_id]

    n_api = NodeApiService.new(@node)
    res = n_api.container()
    if res && res.code == "200"
      @node.update node_status: :healthy
      @containers = JSON.parse(res.body)
      @containers.each { |c| c[:node_id] = @node.id }

      @container_grid = ContainerGrid.new(params[:container_grid]) do |scope|
        scope = @containers
      end
    else
      @node.update node_status: :connection_lost
    end
  end

  def show
    @node = Node.find params[:node_id]

    n_api = NodeApiService.new(@node)
    res = n_api.container_detail(params[:id], "inspect")
    if res && res.code == "200"
      @node.update node_status: :healthy
      @container_detail = JSON.pretty_generate(JSON.parse(res.body))
    else
      @node.update node_status: :connection_lost
      @container_detail = ""
    end
  end

  def logs
    @node = Node.find params[:node_id]

    n_api = NodeApiService.new(@node)
    res = n_api.container_detail(params[:id], "logs")
    if res && res.code == "200"
      @node.update node_status: :healthy
      @container_logs = JSON.parse(res.body)
    else
      @node.update node_status: :connection_lost
      @container_logs = []
    end
  end
end
