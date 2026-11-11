import type { Messages } from "./en";

/** Russian interface text. Keys must match `en.ts`. */
export const ru: Messages = {
  "app.name": "OpenTalk AI",
  "app.skipToContent": "Перейти к основному содержимому",

  "common.copy": "Копировать",
  "common.copied": "Скопировано",
  "common.copyFailed": "Не удалось скопировать",
  "common.demoTag": "Демо",

  "nav.label": "Основная навигация",
  "nav.settingsGroup": "Настройки",
  "nav.live": "Рабочая область",
  "nav.languages": "Языки",
  "nav.providers": "Провайдеры",
  "nav.privacy": "Конфиденциальность",
  "nav.context": "Бизнес-контекст",
  "nav.appearance": "Оформление",
  "nav.rtl": "Проверка RTL",
  "nav.about": "О программе",

  "live.subtitle": "Захватывайте разговор, читайте его на своём языке и получайте варианты ответа.",
  "live.conversation.title": "Разговор",
  "live.languages.automatic": "Автоматически",
  "live.state.label": "Состояние сеанса",
  "live.state.idle": "Ожидание",
  "live.state.listening": "Слушает",
  "live.state.processing": "Обработка",
  "live.state.error": "Ошибка",
  "live.elapsed.label": "Прошло времени",
  "live.elapsed.notRunning": "не запущено",
  "live.audio.title": "Источники звука",
  "live.audio.microphone": "Микрофон",
  "live.audio.system": "Захватывать системный звук",
  "live.audio.start": "Старт",
  "live.audio.starting": "Запуск…",
  "live.audio.startingNote":
    "Запускается захват. Загрузка локальной модели может занять несколько секунд.",
  "live.audio.stop": "Стоп",
  "live.audio.useMicrophone": "Захватывать микрофон",
  "live.audio.defaultMicrophone": "Микрофон по умолчанию",
  "live.audio.capturing": "Идёт захват звука",
  "live.audio.level.microphone": "Уровень микрофона",
  "live.audio.level.system": "Уровень системного звука",
  "live.audio.consent":
    "Прежде чем начать, убедитесь, что все участники разговора согласны на захват звука.",
  "live.error.no_source_selected": "Выберите хотя бы один источник звука.",
  "live.error.invalid_microphone_selection": "Выбранный микрофон недействителен. Выберите другой.",
  "live.error.already_running": "Захват уже идёт.",
  "live.error.permission_denied.microphone":
    "Доступ к микрофону запрещён. Разрешите OpenTalk AI использовать микрофон в настройках конфиденциальности системы и повторите попытку.",
  "live.error.permission_denied.system":
    "Доступ к системному звуку запрещён. В macOS разрешите его в разделе «Конфиденциальность и безопасность», «Запись экрана и системного звука», затем перезапустите приложение.",
  "live.error.device_unavailable.microphone": "Микрофон не найден.",
  "live.error.device_unavailable.system": "Не найден аудиовыход для захвата.",
  "live.error.unsupported": "Этот источник звука не поддерживается в этой системе.",
  "live.error.interrupted": "Захват неожиданно остановился. Возможно, устройство было отключено.",
  "live.error.backend": "Не удалось запустить захват звука.",
  "live.error.unknown": "При захвате звука произошла ошибка.",
  "live.audio.notStored":
    "Звук никогда не сохраняется. Он покидает этот компьютер, только когда включена расшифровка.",
  "live.transcript.empty":
    "Расшифровки пока нет. Речь появляется здесь, когда идёт захват и включена расшифровка.",
  "live.transcription.off": "Расшифровка выключена. Включите её в разделе «Провайдеры».",
  "live.transcription.on":
    "Расшифровка включена: речь из захватываемых источников отправляется в облачный сервис {provider}.",
  "live.transcription.stopped": "Расшифровка остановлена. Захват звука продолжается.",
  "live.error.missing_api_key":
    "Для провайдера расшифровки не сохранён API-ключ. Добавьте его в разделе «Провайдеры».",
  "live.error.authentication_failed":
    "Провайдер расшифровки отклонил API-ключ. Проверьте ключ в разделе «Провайдеры».",
  "live.error.rate_limited":
    "Провайдер расшифровки ограничивает запросы. Подождите немного и повторите попытку.",
  "live.error.unsupported_language": "Провайдер расшифровки не поддерживает выбранный язык речи.",
  "live.error.connection_failed":
    "Не удалось подключиться к провайдеру расшифровки. Проверьте подключение к интернету.",
  "live.error.connection_lost": "Соединение с провайдером расшифровки потеряно.",
  "live.error.timeout": "Провайдер расшифровки не ответил вовремя.",
  "live.error.malformed_response":
    "Провайдер расшифровки прислал ответ, который не удалось разобрать.",
  "live.error.provider": "Провайдер расшифровки сообщил об ошибке.",
  "live.translation.pending": "Переводится…",
  "live.translation.failed": "Не переведено: {reason}",
  "live.translation.check": "Проверьте этот перевод. Возможно, перенесено неверно: {items}",
  "live.reason.missing_api_key": "не сохранён API-ключ.",
  "live.reason.authentication_failed": "провайдер отклонил API-ключ.",
  "live.reason.rate_limited": "провайдер ограничивает запросы или исчерпана квота аккаунта.",
  "live.reason.unsupported_language": "провайдер не поддерживает этот язык.",
  "live.reason.connection_failed": "не удалось связаться с провайдером.",
  "live.reason.timeout": "провайдер не ответил вовремя.",
  "live.reason.malformed_response": "ответ провайдера не удалось разобрать.",
  "live.reason.refused": "провайдер отказался переводить этот текст.",
  "live.reason.overloaded": "речь поступает быстрее, чем её удаётся переводить.",
  "live.reason.provider": "провайдер сообщил об ошибке.",
  "live.reason.unknown": "произошла ошибка.",
  "live.translating.on":
    "Перевод на {language} включён: готовый текст расшифровки отправляется в {provider}.",
  "live.translating.off": "Перевод выключен.",
  "live.demo.toggle": "Показать демо-данные",
  "live.demo.hint":
    "Заполняет панели фиксированным примером текста, чтобы можно было оценить макет.",
  "live.demo.banner":
    "Демо-режим. Текст ниже — фиксированный пример. Звук не захватывается, не расшифровывается и не переводится.",
  "live.translation.title": "Перевод",
  "live.segment.partial": "В процессе",
  "live.source.microphone": "Микрофон",
  "live.source.system": "Системный звук",
  "live.replies.title": "Варианты ответа",
  "live.replies.refresh": "Обновить варианты",
  "live.replies.empty":
    "Вариантов пока нет. Они появляются после реплики собеседника, если варианты ответа включены в разделе «Провайдеры». За вас никогда ничего не отправляется.",
  "live.replies.refreshUnavailable":
    "Обновление доступно, пока идёт захват и включены варианты ответа.",
  "live.replies.pending": "Готовятся новые варианты…",
  "live.replies.outdated": "Эти варианты относятся к более ранней части разговора.",
  "live.replies.failed": "Не удалось подготовить варианты: {reason}",
  "live.replies.dismiss": "Скрыть",
  "live.replies.dismissLabel": "Скрыть ответ: {style}",
  "live.replies.asksForMissingFacts":
    "Этот ответ задаёт вопрос, потому что нужного факта, например цены или даты, нет ни в разговоре, ни в вашем бизнес-контексте.",
  "live.replies.on":
    "Варианты ответа включены: недавний разговор и ваш бизнес-контекст отправляются в {provider}.",
  "live.replies.off": "Варианты ответа выключены.",
  "live.arabic.generatedDiacritics":
    "Огласовки в этом тексте расставлены ИИ и могут быть неверными. Проверьте их, прежде чем на них полагаться.",
  "live.replies.speak": "Озвучить",
  "live.replies.speakLabel": "Озвучить ответ: {style}",
  "live.replies.noVoice": "Для этого языка в системе не установлен голос.",
  "live.translating.onLocal":
    "Перевод на {language} включён; его выполняет {provider} на этом компьютере.",
  "live.replies.onLocal": "Варианты ответа включены; их пишет {provider} на этом компьютере.",
  "live.error.local_only":
    "Включён только локальный режим, а этот сеанс использовал бы облачный сервис. Смените провайдера или выключите локальный режим в разделе «Провайдеры».",
  "live.reason.model_unavailable": "выбранная модель недоступна.",
  "live.transcription.onLocal":
    "Расшифровка включена; её выполняет {provider} на этом компьютере. Звук его не покидает.",
  "live.error.model_unavailable":
    "Выбранная модель недоступна. Выберите модель в разделе «Провайдеры».",
  "live.replies.style.professional": "Деловой",
  "live.replies.style.friendly": "Дружелюбный",
  "live.replies.style.concise": "Краткий",
  "live.replies.meaning": "Смысл",
  "live.replies.edit": "Изменить",
  "live.replies.doneEditing": "Готово",
  "live.replies.editLabel": "Изменить ответ: {style}",

  "languages.intro":
    "Эти настройки сохраняются на этом устройстве. Какие языки реально работают, зависит от провайдера; пока язык не проверен, он не указывается как поддерживаемый.",
  "languages.source": "Язык речи",
  "languages.sourceAuto": "Определять автоматически",
  "languages.target": "Переводить на",
  "languages.reply": "Язык вариантов ответа",
  "languages.tagHint": "Код языка, например en, az, ru или ar-EG.",
  "languages.tagInvalid": "Введите корректный код языка, например en, az, ru или ar-EG.",
  "languages.arabic.title": "Арабский текст",
  "languages.arabic.scope":
    "Применяется, когда язык перевода или ответа — арабский. При выключенном ташкиле огласовки всегда удаляются, даже если провайдер их добавил.",
  "languages.arabic.dialectUnavailable":
    "Диалект можно выбрать, когда доступен провайдер, принимающий такое предпочтение.",
  "languages.arabic.dialectUnverified":
    "Текст на диалекте не проверялся и может смешиваться с литературным арабским; важные тексты стоит показать носителю языка.",
  "languages.arabic.variety": "Вариант языка",
  "languages.arabic.variety.msa": "Современный стандартный арабский",
  "languages.arabic.variety.dialect": "Диалект",
  "languages.arabic.dialectLabel": "Название диалекта",
  "languages.arabic.dialectHint": "Например, египетский или диалект Залива.",
  "languages.arabic.tashkeel": "Ташкиль (огласовки)",
  "languages.arabic.tashkeel.off": "Выключен",
  "languages.arabic.tashkeel.partial": "Частичный",
  "languages.arabic.tashkeel.full": "Полный",
  "languages.arabic.tashkeelWarning":
    "Огласовки расставляет модель, и они могут быть неверными. Проверяйте их, прежде чем на них полагаться.",

  "providers.intro": "OpenTalk AI использует отдельного заменяемого провайдера для каждой задачи.",
  "providers.location.cloud": "Облачный сервис",
  "providers.location.local": "Работает на этом компьютере",
  "providers.disclosure":
    "Когда расшифровка включена, фрагменты захваченного звука, содержащие речь, отправляются по зашифрованному соединению в {provider} и расшифровываются там. Тишина не отправляется. {provider} обрабатывает этот звук по собственным правилам работы с данными.",
  "providers.capability.partial": "Показывает текст, пока человек ещё говорит.",
  "providers.capability.noPartial": "Показывает текст только после того, как человек замолчал.",
  "providers.capability.languageDetection": "Сообщает, какой язык определил.",
  "providers.capability.noLanguageDetection": "Не сообщает, какой язык определил.",
  "providers.capability.languagesUnknown":
    "Не публикует список поддерживаемых языков; неподдерживаемый язык отклоняется при запуске захвата.",
  "providers.capability.languages": "Поддерживаемые языки: {languages}",
  "providers.key.label": "API-ключ",
  "providers.key.saved": "API-ключ сохранён в системном хранилище учётных данных.",
  "providers.key.missing": "API-ключ не сохранён.",
  "providers.key.hint":
    "Ключ хранится в хранилище учётных данных операционной системы и больше не показывается.",
  "providers.key.save": "Сохранить ключ",
  "providers.key.remove": "Удалить ключ",
  "providers.key.error.invalid_key":
    "Этот ключ недействителен. Он не должен быть пустым или содержать пробелы.",
  "providers.key.error.store_unavailable":
    "Не удалось использовать системное хранилище учётных данных.",
  "providers.enable": "Отправлять захваченную речь в {provider} для расшифровки",
  "providers.enableNeedsKey": "Чтобы включить, сначала сохраните API-ключ.",
  "providers.unavailable": "Настройки расшифровки доступны в настольном приложении.",
  "providers.transcription": "Расшифровка",
  "providers.translation": "Перевод",
  "providers.translation.description": "Переводит расшифровку.",
  "providers.translation.disclosure":
    "Когда перевод включён, текст каждого готового фрагмента расшифровки и ваш глоссарий отправляются по зашифрованному соединению в {provider} и переводятся там. {provider} получает указание не сохранять их.",
  "providers.translation.languagesUnknown":
    "Не публикует список поддерживаемых языковых пар, поэтому здесь ни одна не заявлена. Проверяйте результат для своих языков.",
  "providers.translation.checks":
    "Каждый перевод проверяется: если число, ссылка или термин из глоссария пропали, он помечается для вашей проверки.",
  "providers.translation.enable": "Отправлять текст расшифровки в {provider} для перевода",
  "providers.replies": "Варианты ответа",
  "providers.replies.disclosure":
    "Когда варианты ответа включены, недавний разговор (не более 24 последних реплик) и ваш бизнес-контекст отправляются по зашифрованному соединению в {provider}, где и готовятся ответы. {provider} получает указание не сохранять их.",
  "providers.replies.rules":
    "Варианты — это черновики, которые вы читаете, правите и произносите сами. OpenTalk AI никогда ничего не отправляет от вашего имени.",
  "providers.replies.source":
    "Варианты отвечают собеседнику, а его речь захватывается через системный звук.",
  "providers.replies.enable": "Отправлять разговор в {provider} для вариантов ответа",
  "providers.speech": "Озвучивание",
  "providers.speech.systemVoices": "Системные голоса",
  "providers.speech.description":
    "При выборе системных голосов текст произносят голоса, установленные на этом компьютере. Ничего никуда не отправляется.",
  "providers.speech.voices":
    "Какие языки можно озвучить, зависит от голосов, установленных в настройках системы.",
  "providers.speech.enable": "Показывать кнопку «Озвучить» у вариантов ответа",
  "providers.speech.unavailable": "Эта система не предоставляет приложению вывод речи.",
  "providers.speech.provider": "Голос",
  "providers.speech.provider.system": "Системные голоса, на этом компьютере",
  "providers.speech.provider.elevenlabs": "ElevenLabs, облачный сервис",
  "providers.speech.provider.compatible": "Сервис с API озвучивания в стиле OpenAI",
  "providers.speech.cloudDescription":
    "Каждый озвучиваемый текст отправляется по зашифрованному соединению в облачный сервис {provider} и превращается в речь там. Больше ничего не отправляется.",
  "providers.speech.localDescription":
    "Каждый озвучиваемый текст превращает в речь сервер {provider} на этом компьютере. Ничего его не покидает.",
  "providers.speech.model": "Модель озвучивания",
  "providers.speech.elevenlabs.voice": "ID голоса",
  "providers.speech.elevenlabs.voiceHint": "ID голоса из вашей библиотеки голосов ElevenLabs.",
  "providers.speech.elevenlabs.modelHint":
    "От модели зависит, на каких языках можно озвучивать. eleven_v3 говорит более чем на 70, включая азербайджанский.",
  "providers.speech.baseUrlHint":
    "Базовый адрес API, например https://api.openai.com/v1. Он должен начинаться с https, если сервис работает не на этом компьютере.",
  "providers.speech.compatible.modelHint":
    "Как её называет сервис, например gpt-4o-mini-tts у OpenAI.",
  "providers.speech.compatible.voice": "Голос",
  "providers.speech.compatible.voiceHint": "Один из голосов модели, например alloy.",
  "providers.speech.translations.enable": "Озвучивать переводы собеседника",
  "providers.speech.translations.description":
    "Пока идёт захват, каждый готовый перевод того, что слышно в системном звуке, произносится на вашем языке через несколько секунд после сказанного. Используйте наушники, чтобы микрофон не улавливал этот голос.",
  "providers.speech.translations.needsTranslation":
    "Перевод выключен, поэтому озвучивать нечего. Включите его выше.",
  "providers.speech.translations.noVoice":
    "Для языка «{language}» не установлен системный голос. Установите его в настройках системы или выберите другой голос выше.",
  "live.speech.on": "Переводы собеседника озвучивает {provider} на этом компьютере.",
  "live.speech.onCloud":
    "Переводы собеседника озвучиваются: их текст отправляется в облачный сервис {provider}.",
  "live.speech.needsTranslation":
    "Озвучивание переводов включено, но перевод выключен, поэтому ничего не произносится.",
  "live.speech.failed": "Не удалось озвучить перевод: {reason}",
  "live.speech.reason.invalid_settings": "настройки голоса неполные или неверные.",
  "live.speech.reason.local_only": "включён режим «только локально».",
  "providers.localOnly.title": "Только локальный режим",
  "providers.localOnly.toggle": "Никогда не обращаться к облачным сервисам",
  "providers.localOnly.description":
    "Когда режим включён, облачных провайдеров выбрать нельзя, и приложение отказывается начинать сеанс, который отправил бы им какие-либо данные.",
  "providers.localOnly.blocked": "Выключено локальным режимом.",
  "providers.choice.label": "Провайдер",
  "providers.choice.cloud": "{provider} · облачный сервис",
  "providers.choice.local": "{provider} · на этом компьютере",
  "providers.custom.title": "Другой ИИ-сервис",
  "providers.custom.description":
    "Используйте для перевода и вариантов ответа любой сервис с OpenAI-совместимым API, например OpenRouter. Укажите его адрес, название модели и ваш API-ключ для этого сервиса.",
  "providers.custom.baseUrl": "Адрес сервиса",
  "providers.custom.baseUrlHint":
    "Базовый URL API, например https://openrouter.ai/api/v1. Он должен начинаться с https, если сервис работает не на этом компьютере.",
  "providers.custom.baseUrlInvalid": "Введите адрес https или адрес http на этом компьютере.",
  "providers.custom.model": "Название модели",
  "providers.custom.modelHint":
    "Так, как её называет сервис, например openai/gpt-4o-mini в OpenRouter.",
  "providers.custom.policies":
    "Текст, отправленный этому сервису, обрабатывается по его правилам, а сервис-посредник может передать его компании, которая запускает модель.",
  "providers.custom.needsSetup":
    "Сначала укажите адрес сервиса, название модели и API-ключ в разделе «Другой ИИ-сервис».",
  "providers.custom.unnamed": "Другой сервис",
  "providers.local.title": "Локальный ИИ",
  "providers.local.description":
    "Ollama запускает языковые модели на этом компьютере. OpenTalk AI может использовать такую модель для перевода и вариантов ответа, и тогда текст не покидает компьютер.",
  "providers.local.endpoint": "Адрес Ollama",
  "providers.local.endpointHint":
    "Должен быть адресом на этом компьютере, обычно http://localhost:11434.",
  "providers.local.check": "Проверить подключение",
  "providers.local.checking": "Проверка…",
  "providers.local.running": "Ollama {version} работает.",
  "providers.local.model": "Модель",
  "providers.local.chooseModel": "Выберите модель",
  "providers.local.noModels":
    'Модели не установлены. Установите модель сами через Ollama, например выполнив в терминале "ollama pull" и имя модели. OpenTalk AI никогда не скачивает модели.',
  "providers.local.resources":
    "Модели нужно примерно столько свободной памяти, сколько она занимает, а каждый ответ занимает несколько секунд. Качество зависит от модели и отличается для разных языков.",
  "providers.local.error.not_local":
    "Этот адрес не на этом компьютере. Используйте адрес вида http://localhost:11434.",
  "providers.local.error.not_running":
    "По этому адресу сервер Ollama не отвечает. Запустите Ollama и проверьте снова.",
  "providers.local.error.timeout": "Ollama не ответил вовремя.",
  "providers.local.error.malformed_response":
    "По этому адресу что-то ответило, но это не похоже на Ollama.",
  "providers.local.error.server": "Ollama сообщил об ошибке.",
  "providers.local.error.unknown": "Локальный ИИ можно проверить в настольном приложении.",
  "providers.local.processing":
    "Это выполняет модель {model} в Ollama на этом компьютере. По сети ничего не отправляется.",
  "providers.local.needsModel": "Сначала выберите локальную модель в разделе «Локальный ИИ».",
  "providers.translation.needsTranscription":
    "Для перевода нужны включённая расшифровка и настроенный провайдер: сохранённый API-ключ или выбранная локальная модель.",
  "providers.replies.needsTranscription":
    "Для вариантов ответа нужны включённая расшифровка и настроенный провайдер: сохранённый API-ключ или выбранная локальная модель.",
  "providers.whisper.enable":
    "Расшифровывать захваченную речь с помощью {provider} на этом компьютере",
  "providers.whisper.title": "Локальная расшифровка",
  "providers.whisper.description":
    "Whisper — модель распознавания речи, которая работает внутри OpenTalk AI на этом компьютере. Ей нужен только файл модели, другие программы не требуются.",
  "providers.whisper.processing":
    "Речь превращается в текст на этом компьютере моделью Whisper {model}. Звук никуда не отправляется.",
  "providers.whisper.needsModel":
    "Сначала выберите модель Whisper в разделе «Локальная расшифровка».",
  "providers.whisper.finalOnly":
    "Текст появляется, когда человек делает паузу, а не пока он говорит.",
  "providers.whisper.languages":
    "Модели многоязычные, точность зависит от языка. Большие модели точнее и медленнее.",
  "providers.whisper.model": "Модель",
  "providers.whisper.chooseModel": "Выберите модель",
  "providers.whisper.noModels":
    "Модель ещё не установлена. Скачайте одну ниже или положите файл модели Whisper (ggml-*.bin) в папку моделей.",
  "providers.whisper.download": "Скачать модели",
  "providers.whisper.downloadNote":
    "Модель скачивается, только когда вы нажимаете «Скачать», из репозитория моделей whisper.cpp на Hugging Face, и перед использованием сверяется с опубликованной контрольной суммой.",
  "providers.whisper.downloadButton": "Скачать {model} ({size})",
  "providers.whisper.installed": "{model} установлена",
  "providers.whisper.downloading": "Скачивается {model}: {percent}%",
  "providers.whisper.cancel": "Отмена",
  "providers.whisper.folder": "Папка моделей: {folder}",
  "providers.whisper.license":
    "Модели Whisper опубликованы OpenAI под лицензией MIT. Модели нужно примерно столько памяти, сколько занимает её файл.",
  "providers.whisper.error.local_only":
    "В локальном режиме скачивание отключено, потому что оно обращается к серверу загрузки. Чтобы скачать, выключите локальный режим, а затем включите снова, либо скопируйте файл модели в папку моделей.",
  "providers.whisper.error.connection_failed": "Не удалось связаться с сервером загрузки.",
  "providers.whisper.error.cancelled": "Скачивание отменено.",
  "providers.whisper.error.verification_failed":
    "Скачанный файл не совпал с опубликованной моделью и был удалён.",
  "providers.whisper.error.storage_failed": "Не удалось сохранить модель на диск.",
  "providers.whisper.error.busy": "Другое скачивание уже идёт.",
  "providers.whisper.error.unknown_model": "Этой модели нет в списке.",
  "providers.whisper.error.unknown": "Моделями можно управлять в настольном приложении.",
  "providers.custom.service": "Сервис",
  "providers.custom.other": "Другой адрес",
  "providers.localOnly.noTranscription":
    "С моделью Whisper для расшифровки и локальной моделью в Ollama для перевода и ответов всё работает на этом компьютере.",
  "providers.custom.noTranscription":
    "Эти сервисы используются для перевода и вариантов ответа. Для расшифровки выберите OpenAI или встроенную модель Whisper.",
  "live.transcription.localOnly":
    "Включён только локальный режим. Чтобы расшифровывать на этом компьютере, выберите модель Whisper в разделе «Провайдеры».",
  "providers.replies.description": "Готовит ответы, которые вы проверяете сами.",

  "privacy.intro": "Как OpenTalk AI должен обращаться с вашими разговорами.",
  "privacy.principle.capture":
    "Захват выключен, пока вы не нажмёте «Старт», а когда он включён, это видно по индикатору.",
  "privacy.principle.audio": "Исходный звук не сохраняется.",
  "privacy.principle.cloud":
    "Ничего не отправляется облачному провайдеру, пока вы сами его не выберете.",
  "privacy.principle.autosend": "Ответы никогда не отправляются автоматически.",
  "privacy.buildStatus":
    "Звук захватывается только между «Старт» и «Стоп». Речь отправляется провайдеру расшифровки, только если вы включите это в разделе «Провайдеры».",
  "privacy.history.title": "История сеансов",
  "privacy.history.toggle": "Сохранять историю сеансов на этом устройстве",
  "privacy.history.unavailable":
    "История выключена, и включить её пока нельзя: локальное хранение истории ещё не реализовано.",
  "privacy.stored.title": "Что хранится на этом устройстве",
  "privacy.stored.body":
    "Хранятся только настройки с этих экранов (тема, языки, бизнес-контекст и глоссарий) — в локальном хранилище приложения.",
  "privacy.reset": "Сбросить все настройки",
  "privacy.resetDone": "Настройки сброшены.",

  "context.intro":
    "Сведения, которые провайдер ответов может использовать, чтобы варианты подходили вашему бизнесу. Они хранятся на этом устройстве и в этой сборке никуда не отправляются.",
  "context.profile": "О вашем бизнесе",
  "context.profileHint":
    "Услуги, часы работы, цены, которые вы готовы называть. Не более {max} символов.",
  "context.glossary.title": "Глоссарий",
  "context.glossary.intro": "Термины, которые всегда нужно переводить определённым образом.",
  "context.glossary.empty": "Терминов пока нет.",
  "context.glossary.term": "Термин",
  "context.glossary.preferred": "Предпочтительный перевод",
  "context.glossary.entry": "Термин {number}",
  "context.glossary.add": "Добавить термин",
  "context.glossary.remove": "Удалить термин {number}",
  "context.glossary.limit": "Достигнут предел в {max} терминов.",

  "appearance.theme": "Тема",
  "appearance.theme.system": "Как в системе",
  "appearance.theme.light": "Светлая",
  "appearance.theme.dark": "Тёмная",
  "appearance.language": "Язык интерфейса",

  "rtl.intro":
    "Проверяет, что арабский и другой текст с письмом справа налево отображается правильно. Арабский текст ниже — фиксированный пример, а не результат работы провайдера.",
  "rtl.toggle": "Показать весь интерфейс справа налево",
  "rtl.samples": "Пример текста",
  "rtl.sample.plain": "Без ташкиля",
  "rtl.sample.tashkeel": "С полным ташкилем",
  "rtl.sample.mixed": "Арабский с латиницей и числами",
  "rtl.sample.url": "Арабский со ссылкой",

  "about.description":
    "Настольный помощник с открытым исходным кодом для многоязычных разговоров в реальном времени.",
  "about.status":
    "Сборка для разработки. Расшифровка, перевод и варианты ответа пока не проверялись на реальном сервисе.",
  "about.core.loading": "Подключение к ядру приложения…",
  "about.core.ready": "Ядро приложения: {name} {version}",
  "about.core.unavailable":
    "Ядро приложения недоступно. Чтобы подключиться, запустите приложение через Tauri.",
  "about.license": "Лицензия: MIT",
};
