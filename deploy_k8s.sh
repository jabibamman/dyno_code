#!/bin/sh

project_id=pa2024-421814

read -p "Souhaitez-vous déployer dyno-code ? (y/n) " deploy_dyno_code
read -p "Souhaitez-vous déployer executor ? (y/n) " deploy_executor

if [ "$deploy_dyno_code" = "y" ] || [ "$deploy_dyno_code" = "Y" ]; then
  docker build -t dyno_code_api/dyno-code:latest .
  docker tag dyno_code_api/dyno-code:latest gcr.io/$project_id/dyno-code:latest
  docker push gcr.io/$project_id/dyno-code:latest
fi

if [ "$deploy_executor" = "y" ] || [ "$deploy_executor" = "Y" ]; then
  docker build -f Dockerfile.executor -t gcr.io/$project_id/executor:latest .
  docker push gcr.io/$project_id/executor:latest
fi

kubectl apply -f deployment.yaml
kubectl apply -f service.yaml
kubectl apply -f role_binding.yaml

if [ "$deploy_dyno_code" = "y" ] || [ "$deploy_dyno_code" = "Y" ]; then
  kubectl set image deployment/dyno-code-deployment dyno-code=gcr.io/$project_id/dyno-code:latest
  kubectl rollout restart deployment/dyno-code-deployment
fi
