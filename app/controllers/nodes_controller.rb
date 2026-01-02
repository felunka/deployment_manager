class NodesController < ApplicationController
  def index
    @nodes_grid = NodesGrid.new(params[:nodes_grid]) do |scope|
      scope.page(params[:page])
    end
  end

  def show
    @node = Node.find_by params.permit(:id)

    unless @node.healthy? || @node.pending_init?
      Thread.new do
        n_api = NodeApiService.new(@node)
        res = n_api.health()
        if res && res.code == "200"
          @node.update node_status: :healthy
          @health_data = JSON.parse(res.body)
        end
      end
    end
  end

  def health
    @node = Node.find_by params.permit(:id)

    n_api = NodeApiService.new(@node)
    res = n_api.health()
    if res && res.code == "200"
      @node.update node_status: :healthy
      @health_data = JSON.parse(res.body)
    else
      @node.update node_status: :connection_lost
      @health_data = JSON.parse('{
        "memory_total": -1,
        "memory_swapped": -1,
        "memory_free": -1,
        "memory_buffer": -1,
        "memory_cache": -1,
        "io_bytes_in": -1,
        "io_bytes_out": -1,
        "cpu_usage": -1
      }')
    end
  end

  def new
    @node = Node.new
  end

  def create
    @node = Node.new permit(params)

    respond_to do |format|
      if @node.save
        # Start adopt node process
        Thread.new do
          node_api = NodeApiService.new(@node)
          20.times do |n|
            response = node_api.health()
            if response.code == "200"
              @node.update node_status: :healthy
              break
            end
            sleep 10
          end

          @node.update node_status: :init_failed unless @node.healthy?
        end

        flash[:success] = t("messages.model.created")
        format.html { redirect_to action: "show", id: @node.id }
      else
        format.html { render :new, status: :unprocessable_entity }
      end
    end
  end

  def edit
    @node = Node.find_by params.permit(:id)
  end

  def update
    @node = Node.find_by params.permit(:id)

    respond_to do |format|
      if @node.update permit(params)
        format.html { redirect_to action: "show", id: @node.id }
      else
        format.html { render :edit, status: :unprocessable_entity }
      end
    end
  end

  def destroy
    Node.find_by(params.permit(:id)).destroy
    flash[:danger] = t("messages.model.deleted")
    redirect_to action: "index"
  end

  private

  def permit(params)
    params.require(:node).permit(
      :hostname,
      :ip,
      :api_url,
      :port,
      :key,
    )
  end
end
