class NodeDeploymentsController < ApplicationController
  def index
    if params[:node_id].present?
      @node = Node.find params[:node_id]
    end

    @node_deployments_grid = NodeDeploymentsGrid.new(params[:node_deployments_grid]) do |scope|
      if @node.present?
        scope.where(node: @node).page(params[:page])
      else
        scope.page(params[:page])
      end
    end
  end

  def show
    @node_deployment = NodeDeployment.find_by params.permit(:id)
  end

  def status
    @node_deployment = NodeDeployment.find_by params.permit(:id)

    n_api = NodeApiService.new(@node_deployment.node)
    if @node_deployment.simple_docker_run?
      res = n_api.container_detail(params[:id], "logs")
    elsif @node_deployment.simple_docker_compose?
      res = n_api.compose_logs(@node_deployment)
    elsif @node_deployment.github_action_runner?
      res = n_api.runner_status(@node_deployment)
    end

    if res && res.code == "200"
      @node_deployment.update deployment_status: :healthy

      if @node_deployment.github_action_runner?
        @status = res.body
      else
        @status = JSON.parse(res.body)
      end
    else
      @node_deployment.update deployment_status: :connection_lost

      @status = ""
    end
  end

  def new
    @node_deployment = NodeDeployment.new

    if params[:node_id].present?
      @node = Node.find params[:node_id]
      @node_deployment.node = @node
    end
  end

  def create
    @node_deployment = NodeDeployment.new permit(params)

    respond_to do |format|
      if @node_deployment.save
        # Start setup deployment
        Thread.new do
          setup_deployment(@node_deployment, params.require(:node_deployment).permit(:github_token, :adopt))
        end

        flash[:success] = t("messages.model.created")
        format.html { redirect_to action: "show", id: @node_deployment.id }
      else
        format.html { render :new, status: :unprocessable_entity }
      end
    end
  end

  def edit
    @node_deployment = NodeDeployment.find_by params.permit(:id)
  end

  def update
    @node_deployment = NodeDeployment.find_by params.permit(:id)

    respond_to do |format|
      if @node_deployment.update permit(params)
        # Update deployment data
        Thread.new do
          update_deployment(@node_deployment)
        end
        format.html { redirect_to action: "show", id: @node_deployment.id }
      else
        format.html { render :edit, status: :unprocessable_entity }
      end
    end
  end

  def destroy
    NodeDeployment.find_by(params.permit(:id)).destroy
    flash[:danger] = t("messages.model.deleted")
    redirect_to action: "index"
  end

  private

  def permit(params)
    params.require(:node_deployment).permit(
      :node_id,
      :name,
      :path,
      :git_url,
      :deployment_type,
      :compose,
    )
  end

  def setup_deployment(node_deployment, permitted_params)
    node_api = NodeApiService.new(node_deployment.node)
    # Test if node healthy
    response = node_api.health()
    if response && response.code == "200"

      unless permitted_params[:adopt] == "1"
        if node_deployment.simple_docker_run?
          response = node_api.container_create(node_deployment)
        elsif node_deployment.simple_docker_compose?
          response = node_api.setup_compose(node_deployment)
        elsif node_deployment.github_action_runner?
          response = node_api.setup_runner(node_deployment, permitted_params[:github_token])
        end
        if response && response.code == "200"
          logger.info(response.body)
          node_deployment.update deployment_status: :healthy
        else
          logger.warn("Deployment failed!")
          logger.warn(response.body)
          node_deployment.update deployment_status: :init_failed
        end
      else
        node_deployment.update deployment_status: :healthy
      end

    else
      logger.warn("Deployment failed! Node not healthy")
      node_deployment.update deployment_status: :init_failed
    end
  end

  def update_deployment(node_deployment)
    node_api = NodeApiService.new(node_deployment.node)
    # Test if node healthy
    response = node_api.health()
    if response && response.code == "200"

      unless permitted_params[:adopt] == "1"
        if node_deployment.simple_docker_run?
          response = node_api.container_create(node_deployment)
        elsif node_deployment.simple_docker_compose?
          response = node_api.setup_compose(node_deployment)
        elsif node_deployment.github_action_runner?
          return
        end
        if response && response.code == "200"
          logger.info(response.body)
          node_deployment.update deployment_status: :healthy
        else
          logger.warn("Deployment update failed!")
          logger.warn(response.body)
          node_deployment.update deployment_status: :init_failed
        end
      else
        node_deployment.update deployment_status: :healthy
      end

    else
      logger.warn("Deployment update failed! Node not healthy")
      node_deployment.update deployment_status: :init_failed
    end
  end
end
