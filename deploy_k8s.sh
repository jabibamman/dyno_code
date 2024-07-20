#!/bin/sh

project_id=pa2024-428618
arch=$(uname -m)

if [ "$arch" = "x86_64" ]; then
    platform="linux/amd64"
elif [ "$arch" = "arm64" ]; then
    platform="linux/arm64"
else
    echo "Unsupported architecture: $arch"
    exit 1
fi

read -p "Souhaitez-vous déployer dyno-code ? (y/n) " deploy_dyno_code
read -p "Souhaitez-vous déployer executor ? (y/n) " deploy_executor
versions_to_keep=2

if [ "$deploy_executor" = "y" ] || [ "$deploy_executor" = "Y" ]; then
  read -p "Souhaitez-vous déployer tous les langages ? (y/n) " deploy_all_languages

  if [ "$deploy_all_languages" = "y" ] || [ "$deploy_all_languages" = "Y" ]; then
    deploy_rust="y"
    deploy_python="y"
    deploy_nodejs="y"
    deploy_lua="y"
  else 
    read -p "Souhaitez-vous déployer Rust ? (y/n) " deploy_rust
    read -p "Souhaitez-vous déployer Python ? (y/n) " deploy_python
    read -p "Souhaitez-vous déployer Node.js ? (y/n) " deploy_nodejs
    read -p "Souhaitez-vous déployer Lua ? (y/n) " deploy_lua
  fi
fi

clean_old_images() {
  local image_name=$1
  local project_id=$2

  tags=$(gcloud container images list-tags gcr.io/$project_id/$image_name --format='get(tags)' --limit=$versions_to_keep --sort-by=TIMESTAMP --filter='-tags:*' --format='value(tags)')

  for tag in $tags; do
    gcloud container images delete --quiet gcr.io/$project_id/$image_name:$tag
  done
}

if [ "$deploy_dyno_code" = "y" ] || [ "$deploy_dyno_code" = "Y" ]; then
  docker buildx build --platform $platform -t gcr.io/$project_id/dyno-code:latest --push .
  clean_old_images "dyno-code" "$project_id"
fi

if [ "$deploy_executor" = "y" ] || [ "$deploy_executor" = "Y" ]; then
  if [ "$deploy_rust" = "y" ] || [ "$deploy_rust" = "Y" ]; then
    docker buildx build --platform $platform -f languages/Dockerfile.rust -t gcr.io/$project_id/executor-rust:latest --push .
  fi

  if [ "$deploy_python" = "y" ] || [ "$deploy_python" = "Y" ]; then
    docker buildx build --platform $platform  -f languages/Dockerfile.python -t gcr.io/$project_id/executor-python:latest --push .
  fi

  if [ "$deploy_nodejs" = "y" ] || [ "$deploy_nodejs" = "Y" ]; then
    docker buildx build --platform $platform -f languages/Dockerfile.nodejs -t gcr.io/$project_id/executor-nodejs:latest --push .
  fi

  if [ "$deploy_lua" = "y" ] || [ "$deploy_lua" = "Y" ]; then
    docker buildx build --platform $platform -f languages/Dockerfile.lua -t gcr.io/$project_id/executor-lua:latest --push .
  fi

  clean_old_images "executor" "$project_id"
fi

kubectl apply -f deployment.yaml
kubectl apply -f service.yaml
kubectl apply -f role_binding.yaml
kubectl apply -f ingress.yaml
kubectl apply -f nfs-server-service.yaml
kubectl apply -f nfs-server.yaml
kubectl apply -f pv-pvc-nfs.yaml
#kubectl apply -f pv-pvc.yaml # Uncomment this line if you want to use local storage instead of NFS

if [ "$deploy_dyno_code" = "y" ] || [ "$deploy_dyno_code" = "Y" ]; then
  kubectl set image deployment/dyno-code-deployment dyno-code=gcr.io/$project_id/dyno-code:latest
  kubectl rollout restart deployment/dyno-code-deployment
fi
