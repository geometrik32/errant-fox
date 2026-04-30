/* 
    Noodl Bridge for Errant Fox 
    Этот файл связывает ваш UI в Noodl с бэкендом на Rust.
*/

import * as Proto3 from './protobuf-bundle.js'; // Предполагается, что вы сгенерируете bundle или импортируете части

export class ErrantFoxAPI {
    constructor(config) {
        this.wsUrl = config.wsUrl || "ws://localhost:8095/api/ws";
        this.socket = null;
        this.onMessageCallback = null;
        this.status = "disconnected";
    }

    // Подключение к серверу
    connect() {
        this.socket = new WebSocket(this.wsUrl);
        
        this.socket.onopen = () => {
            this.status = "connected";
            console.log("Connected to Errant Fox Backend");
        };

        this.socket.onmessage = (event) => {
            const data = JSON.parse(event.data);
            if (this.onMessageCallback) this.onMessageCallback(data);
        };

        this.socket.onclose = () => {
            this.status = "disconnected";
            setTimeout(() => this.connect(), 3000); // Авто-реконнект
        };
    }

    // Отправка команды на сервер
    send(cmd, payload) {
        if (this.status !== "connected") return;
        const message = JSON.stringify({ [cmd]: payload });
        this.socket.send(message);
    }

    // --- УДОБНЫЕ МЕТОДЫ ДЛЯ NOODL ---

    // Войти в систему
    login(username, password) {
        this.send("login", { username, password });
    }

    // Получить список видео/папок
    openPage(pageId) {
        this.send("openNavigationPage", { pageId: pageId || "" });
    }

    // Открыть конкретное видео
    openVideo(videoId) {
        this.send("openMediaFile", { mediaFileId: videoId });
    }

    // Оставить комментарий
    addComment(videoId, text, timecode = "") {
        this.send("addComment", {
            mediaFileId: videoId,
            comment: text,
            timecode: timecode
        });
    }
}
