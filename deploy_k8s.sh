#!/bin/sh

project_id=pa2024-421814

read -p "Souhaitez-vous déployer dyno-code ? (y/n) " deploy_dyno_code
read -p "Souhaitez-vous déployer executor ? (y/n) " deploy_executor

versions_to_keep=2

clean_old_images() {
  local image_name=$1
  local project_id=$2

  tags=$(gcloud container images list-tags gcr.io/$project_id/$image_name --format='get(tags)' --limit=$versions_to_keep --sort-by=TIMESTAMP --filter='-tags:*' --format='value(tags)')

  for tag in $tags; do
    gcloud container images delete --quiet gcr.io/$project_id/$image_name:$tag
  done
}

if [ "$deploy_dyno_code" = "y" ] || [ "$deploy_dyno_code" = "Y" ]; then
  docker build -t dyno_code_api/dyno-code:latest .
  docker tag dyno_code_api/dyno-code:latest gcr.io/$project_id/dyno-code:latest
  docker push gcr.io/$project_id/dyno-code:latest

  clean_old_images "dyno-code" "$project_id"
fi

if [ "$deploy_executor" = "y" ] || [ "$deploy_executor" = "Y" ]; then
  docker build -f Dockerfile.executor -t gcr.io/$project_id/executor:latest .
  docker push gcr.io/$project_id/executor:latest

  clean_old_images "executor" "$project_id"
fi

kubectl apply -f deployment.yaml
kubectl apply -f service.yaml
kubectl apply -f role_binding.yaml
kubectl apply -f ingress.yaml
kubectl apply -f pv-pvc.yaml

if [ "$deploy_dyno_code" = "y" ] || [ "$deploy_dyno_code" = "Y" ]; then
  kubectl set image deployment/dyno-code-deployment dyno-code=gcr.io/$project_id/dyno-code:latest
  kubectl rollout restart deployment/dyno-code-deployment
fi
