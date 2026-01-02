require "net/http"

class NodeApiService
  def initialize(node)
    @node = node
  end

  def health
    get("health")
  end

  def create_runner(node_deployment, token)
    body = {
      token: token,
      path: node_deployment.path,
      git_url: node_deployment.git_url
    }

    post("runner", body)
  end

  # Docker
  def container
    get("docker/containers/list")
  end

  def container_detail(id, action)
    get("docker/container/#{id}/#{action}")
  end

  # Compose
  def setup_compose(node_deployment)
    body = {
      path: node_deployment.path,
      compose: node_deployment.compose
    }

    post("docker/compose", body)
  end

  def compose_logs(node_deployment)
    get("docker/compose/status?path=#{CGI.escape(node_deployment.path)}")
  end

  # Runner
  def setup_runner(node_deployment, token)
    body = {
      token: token,
      path: node_deployment.path,
      git_url: node_deployment.git_url
    }

    post("runner", body)
  end

  def runner_status(node_deployment)
    get("runner/status?path=#{CGI.escape(node_deployment.path)}")
  end


  private

  def get(endpoint)
    uri = URI("#{@node.api_url}/#{endpoint}")
    http = Net::HTTP.new(uri.host, uri.port)
    http.use_ssl = (uri.scheme == "https")
    http.verify_mode = OpenSSL::SSL::VERIFY_NONE if Rails.env.development?

    request = Net::HTTP::Get.new(uri.request_uri, { "Content-Type": "application/json", "X-Api-Key": @node.key, "Host": uri.host })

    begin
      http.request(request)
    rescue
      false
    end
  end

  def post(endpoint, body)
    uri = URI("#{@node.api_url}/#{endpoint}")
    http = Net::HTTP.new(uri.host, uri.port)
    http.use_ssl = (uri.scheme == "https")
    http.verify_mode = OpenSSL::SSL::VERIFY_NONE if Rails.env.development?

    request = Net::HTTP::Post.new(uri.request_uri, { "Content-Type": "application/json", "X-Api-Key": @node.key, "Host": uri.host })
    request.body = body.to_json

    begin
      http.request(request)
    rescue Exception => e
      logger.warn(e)
      false
    end
  end
end
