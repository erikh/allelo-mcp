test: start-ollama
	cargo test -- --nocapture
	make stop-ollama

start-ollama: stop-ollama
	docker run --privileged -d -v ${HOME}/.ollama:/root/.ollama --net host --name ollama ollama/ollama >.docker-ollama
	sleep 5
	docker run -it --net host ollama/ollama pull vicuna:7b

stop-ollama:
	if [ -f .docker-ollama ]; then docker rm -f `cat .docker-ollama`; fi
	rm -f .docker-ollama
