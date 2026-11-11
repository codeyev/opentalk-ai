/**
 * English interface text. This file is the source of truth: its keys define
 * `MessageKey`, and every other locale must provide exactly the same keys.
 * `{name}` marks a placeholder filled in at runtime.
 */
export const en = {
  "app.name": "OpenTalk AI",
  "app.skipToContent": "Skip to main content",

  "common.copy": "Copy",
  "common.copied": "Copied",
  "common.copyFailed": "Could not copy",
  "common.demoTag": "Demo",

  "nav.label": "Main navigation",
  "nav.settingsGroup": "Settings",
  "nav.live": "Live workspace",
  "nav.languages": "Languages",
  "nav.providers": "Providers",
  "nav.privacy": "Privacy",
  "nav.context": "Business context",
  "nav.appearance": "Appearance",
  "nav.rtl": "RTL preview",
  "nav.about": "About",

  "live.subtitle": "Capture a conversation, read it in your language, and get reply suggestions.",
  "live.conversation.title": "Conversation",
  "live.languages.automatic": "Automatic",
  "live.state.label": "Session state",
  "live.state.idle": "Idle",
  "live.state.listening": "Listening",
  "live.state.processing": "Processing",
  "live.state.error": "Error",
  "live.elapsed.label": "Elapsed time",
  "live.elapsed.notRunning": "not running",
  "live.audio.title": "Audio sources",
  "live.audio.microphone": "Microphone",
  "live.audio.system": "Capture system audio",
  "live.audio.start": "Start",
  "live.audio.starting": "Starting…",
  "live.audio.startingNote": "Starting capture. Loading a local model can take a few seconds.",
  "live.audio.stop": "Stop",
  "live.audio.useMicrophone": "Capture microphone",
  "live.audio.defaultMicrophone": "System default",
  "live.audio.capturing": "Capturing audio",
  "live.audio.level.microphone": "Microphone level",
  "live.audio.level.system": "System audio level",
  "live.audio.consent":
    "Before you start, make sure everyone in the conversation agrees to being captured.",
  "live.error.no_source_selected": "Choose at least one audio source.",
  "live.error.invalid_microphone_selection":
    "The selected microphone is not valid. Choose another one.",
  "live.error.already_running": "Capture is already running.",
  "live.error.permission_denied.microphone":
    "Microphone access was denied. Allow OpenTalk AI to use the microphone in your system's privacy settings, then try again.",
  "live.error.permission_denied.system":
    "System audio access was denied. On macOS, allow it under Privacy & Security, Screen & System Audio Recording, then restart the app.",
  "live.error.device_unavailable.microphone": "No microphone was found.",
  "live.error.device_unavailable.system": "No audio output was found to capture.",
  "live.error.unsupported": "This audio source is not supported on this system.",
  "live.error.interrupted": "Capture stopped unexpectedly. A device may have been disconnected.",
  "live.error.backend": "Audio capture could not be started.",
  "live.error.unknown": "Something went wrong with audio capture.",
  "live.audio.notStored":
    "Audio is never saved. It leaves this computer only when transcription is turned on.",
  "live.transcript.empty":
    "No transcript yet. Speech appears here while capture is running and transcription is on.",
  "live.transcription.off": "Transcription is off. Turn it on under Providers.",
  "live.transcription.on":
    "Transcription is on: speech from the captured sources is sent to {provider}, a cloud service.",
  "live.transcription.stopped": "Transcription has stopped. Audio capture continues.",
  "live.error.missing_api_key":
    "No API key is saved for the transcription provider. Add one under Providers.",
  "live.error.authentication_failed":
    "The transcription provider rejected the API key. Check the key under Providers.",
  "live.error.rate_limited":
    "The transcription provider is limiting requests. Wait a moment and try again.",
  "live.error.unsupported_language":
    "The transcription provider does not support the selected spoken language.",
  "live.error.connection_failed":
    "Could not connect to the transcription provider. Check your internet connection.",
  "live.error.connection_lost": "The connection to the transcription provider was lost.",
  "live.error.timeout": "The transcription provider did not respond in time.",
  "live.error.malformed_response":
    "The transcription provider sent a response that could not be understood.",
  "live.error.provider": "The transcription provider reported an error.",
  "live.translation.pending": "Translating…",
  "live.translation.failed": "Not translated: {reason}",
  "live.translation.check":
    "Check this translation. These may not have been carried over correctly: {items}",
  "live.reason.missing_api_key": "no API key is saved.",
  "live.reason.authentication_failed": "the provider rejected the API key.",
  "live.reason.rate_limited": "the provider is limiting requests, or the account is out of quota.",
  "live.reason.unsupported_language": "the provider does not support this language.",
  "live.reason.connection_failed": "the provider could not be reached.",
  "live.reason.timeout": "the provider did not respond in time.",
  "live.reason.malformed_response": "the provider's reply could not be understood.",
  "live.reason.refused": "the provider declined to translate this text.",
  "live.reason.overloaded": "speech is arriving faster than it can be translated.",
  "live.reason.provider": "the provider reported an error.",
  "live.reason.unknown": "something went wrong.",
  "live.translating.on":
    "Translation into {language} is on: finished transcript text is sent to {provider}.",
  "live.translating.off": "Translation is off.",
  "live.demo.toggle": "Show demo data",
  "live.demo.hint": "Fills the panels with fixed sample text so the layout can be reviewed.",
  "live.demo.banner":
    "Demo mode. The text below is fixed sample data. No audio is being captured, transcribed or translated.",
  "live.translation.title": "Translation",
  "live.segment.partial": "In progress",
  "live.source.microphone": "Microphone",
  "live.source.system": "System audio",
  "live.replies.title": "Reply suggestions",
  "live.replies.refresh": "Refresh suggestions",
  "live.replies.empty":
    "No suggestions yet. They appear after the other person speaks, when reply suggestions are turned on under Providers. Nothing is ever sent for you.",
  "live.replies.refreshUnavailable":
    "Refresh is available while capture is running with reply suggestions turned on.",
  "live.replies.pending": "Preparing new suggestions…",
  "live.replies.outdated": "These suggestions are from earlier in the conversation.",
  "live.replies.failed": "Suggestions could not be prepared: {reason}",
  "live.replies.dismiss": "Dismiss",
  "live.replies.dismissLabel": "Dismiss the {style} reply",
  "live.replies.asksForMissingFacts":
    "This reply asks a question because a fact it would need, such as a price or a date, is not in the conversation or your business context.",
  "live.replies.on":
    "Reply suggestions are on: the recent conversation and your business context are sent to {provider}.",
  "live.replies.off": "Reply suggestions are off.",
  "live.arabic.generatedDiacritics":
    "The diacritics in this text were added by AI and can be wrong. Check them before relying on them.",
  "live.replies.speak": "Speak",
  "live.replies.speakLabel": "Speak the {style} reply",
  "live.replies.noVoice": "No system voice is installed for this language.",
  "live.translating.onLocal":
    "Translation into {language} is on, done by {provider} on this computer.",
  "live.replies.onLocal": "Reply suggestions are on, written by {provider} on this computer.",
  "live.error.local_only":
    "Local-only mode is on, and this session would use a cloud service. Change the provider or turn local-only mode off under Providers.",
  "live.reason.model_unavailable": "the selected model is not available.",
  "live.transcription.onLocal":
    "Transcription is on, done by {provider} on this computer. No audio leaves it.",
  "live.error.model_unavailable":
    "The selected model is not available. Choose a model under Providers.",
  "live.replies.style.professional": "Professional",
  "live.replies.style.friendly": "Friendly",
  "live.replies.style.concise": "Concise",
  "live.replies.meaning": "Meaning",
  "live.replies.edit": "Edit",
  "live.replies.doneEditing": "Done",
  "live.replies.editLabel": "Edit the {style} reply",

  "languages.intro":
    "These preferences are saved on this device. Which languages actually work depends on the provider; none is listed as supported until it has been tested.",
  "languages.source": "Spoken language",
  "languages.sourceAuto": "Detect automatically",
  "languages.target": "Translate into",
  "languages.reply": "Suggest replies in",
  "languages.tagHint": "Language tag, for example en, az, ru or ar-EG.",
  "languages.tagInvalid": "Enter a valid language tag, for example en, az, ru or ar-EG.",
  "languages.arabic.title": "Arabic output",
  "languages.arabic.scope":
    "Applies when the translation or reply language is Arabic. With tashkeel off, diacritics are always removed, even if the provider adds them.",
  "languages.arabic.dialectUnavailable":
    "A dialect can be chosen once a provider that accepts a dialect preference is available.",
  "languages.arabic.dialectUnverified":
    "Dialect output has not been verified and may mix with Standard Arabic; have a native speaker review anything important.",
  "languages.arabic.variety": "Variety",
  "languages.arabic.variety.msa": "Modern Standard Arabic",
  "languages.arabic.variety.dialect": "Dialect",
  "languages.arabic.dialectLabel": "Dialect name",
  "languages.arabic.dialectHint": "For example Egyptian or Gulf.",
  "languages.arabic.tashkeel": "Tashkeel (diacritics)",
  "languages.arabic.tashkeel.off": "Off",
  "languages.arabic.tashkeel.partial": "Partial",
  "languages.arabic.tashkeel.full": "Full",
  "languages.arabic.tashkeelWarning":
    "Diacritics are generated by a model and can be wrong. Review them before relying on them.",

  "providers.intro": "OpenTalk AI uses a separate, replaceable provider for each task.",
  "providers.location.cloud": "Cloud service",
  "providers.location.local": "Runs on this computer",
  "providers.disclosure":
    "When transcription is on, the parts of the captured audio that contain speech are sent over an encrypted connection to {provider} and transcribed there. Silence is not sent. {provider} handles that audio under its own data policies.",
  "providers.capability.partial": "Shows text while someone is still speaking.",
  "providers.capability.noPartial": "Shows text only after someone stops speaking.",
  "providers.capability.languageDetection": "Reports the language it detected.",
  "providers.capability.noLanguageDetection": "Does not report which language it detected.",
  "providers.capability.languagesUnknown":
    "Does not publish a list of supported languages; an unsupported language is rejected when capture starts.",
  "providers.capability.languages": "Supported languages: {languages}",
  "providers.key.label": "API key",
  "providers.key.saved": "An API key is saved in the system credential store.",
  "providers.key.missing": "No API key is saved.",
  "providers.key.hint":
    "The key is kept in the operating system's credential store and is never shown again.",
  "providers.key.save": "Save key",
  "providers.key.remove": "Remove key",
  "providers.key.error.invalid_key":
    "That key is not valid. It must not be empty or contain spaces.",
  "providers.key.error.store_unavailable": "The system credential store could not be used.",
  "providers.enable": "Send captured speech to {provider} for transcription",
  "providers.enableNeedsKey": "Save an API key to turn this on.",
  "providers.unavailable": "Transcription settings are available in the desktop app.",
  "providers.transcription": "Transcription",
  "providers.translation": "Translation",
  "providers.translation.description": "Translates the transcript.",
  "providers.translation.disclosure":
    "When translation is on, the text of each finished transcript segment and your glossary are sent over an encrypted connection to {provider} and translated there. {provider} is asked not to store them.",
  "providers.translation.languagesUnknown":
    "Does not publish a list of supported language pairs, so none is claimed here. Check the result for the languages you use.",
  "providers.translation.checks":
    "Each translation is checked: if a number, link or glossary term from the original is missing, it is marked for you to verify.",
  "providers.translation.enable": "Send transcript text to {provider} for translation",
  "providers.replies": "Reply suggestions",
  "providers.replies.disclosure":
    "When reply suggestions are on, the recent conversation (up to the last 24 things said) and your business context are sent over an encrypted connection to {provider}, which drafts the replies. {provider} is asked not to store them.",
  "providers.replies.rules":
    "Suggestions are drafts for you to read, edit and say yourself. OpenTalk AI never sends anything on your behalf.",
  "providers.replies.source":
    "Suggestions answer the other side of the conversation, which is what system audio captures.",
  "providers.replies.enable": "Send the conversation to {provider} for reply suggestions",
  "providers.speech": "Speech",
  "providers.speech.systemVoices": "System voices",
  "providers.speech.description":
    "With system voices, text is spoken by the voices installed on this computer. Nothing is sent anywhere.",
  "providers.speech.voices":
    "Which languages can be spoken depends on the voices installed in your system settings.",
  "providers.speech.enable": "Show a Speak button on reply suggestions",
  "providers.speech.unavailable": "This system does not offer speech output to the app.",
  "providers.speech.provider": "Voice",
  "providers.speech.provider.system": "System voices, on this computer",
  "providers.speech.provider.elevenlabs": "ElevenLabs, a cloud service",
  "providers.speech.provider.compatible": "A service with an OpenAI-style speech API",
  "providers.speech.cloudDescription":
    "Each text that is read aloud is sent over an encrypted connection to {provider}, a cloud service, and turned into speech there. Nothing else is sent.",
  "providers.speech.localDescription":
    "Each text that is read aloud is turned into speech by the server at {provider} on this computer. Nothing leaves it.",
  "providers.speech.model": "Speech model",
  "providers.speech.elevenlabs.voice": "Voice ID",
  "providers.speech.elevenlabs.voiceHint": "The ID of a voice in your ElevenLabs voice library.",
  "providers.speech.elevenlabs.modelHint":
    "Which languages can be spoken depends on the model. eleven_v3 speaks more than 70, Azerbaijani among them.",
  "providers.speech.baseUrlHint":
    "The API base URL, for example https://api.openai.com/v1. It must start with https, unless the service runs on this computer.",
  "providers.speech.compatible.modelHint":
    "As the service names it, for example gpt-4o-mini-tts on OpenAI.",
  "providers.speech.compatible.voice": "Voice",
  "providers.speech.compatible.voiceHint": "One of the model's voices, for example alloy.",
  "providers.speech.translations.enable": "Read translations of the other side aloud",
  "providers.speech.translations.description":
    "While capture is running, each finished translation of what system audio captures is spoken in your language, a few seconds after it was said. Use headphones, so the microphone does not pick the voice up.",
  "providers.speech.translations.needsTranslation":
    "Translation is off, so there is nothing to read aloud. Turn it on above.",
  "providers.speech.translations.noVoice":
    "No system voice is installed for {language}. Install one in the system settings, or choose another voice above.",
  "live.speech.on": "Translations of the other side are read aloud by {provider} on this computer.",
  "live.speech.onCloud":
    "Translations of the other side are read aloud: their text is sent to {provider}, a cloud service.",
  "live.speech.needsTranslation":
    "Reading translations aloud is on, but translation is off, so nothing is spoken.",
  "live.speech.failed": "A translation could not be read aloud: {reason}",
  "live.speech.reason.invalid_settings": "the voice settings are incomplete or not valid.",
  "live.speech.reason.local_only": "local-only mode is on.",
  "providers.localOnly.title": "Local-only mode",
  "providers.localOnly.toggle": "Never contact a cloud service",
  "providers.localOnly.description":
    "With this on, cloud providers cannot be chosen, and the app refuses to start a session that would send anything to one.",
  "providers.localOnly.blocked": "Turned off by local-only mode.",
  "providers.choice.label": "Provider",
  "providers.choice.cloud": "{provider} · cloud service",
  "providers.choice.local": "{provider} · on this computer",
  "providers.custom.title": "Other AI service",
  "providers.custom.description":
    "Use any service that offers an OpenAI-compatible API, such as OpenRouter, for translation and reply suggestions. Enter its address, a model name, and your API key for that service.",
  "providers.custom.baseUrl": "Service address",
  "providers.custom.baseUrlHint":
    "The API base URL, for example https://openrouter.ai/api/v1. It must start with https, unless the service runs on this computer.",
  "providers.custom.baseUrlInvalid": "Enter an https address, or an http address on this computer.",
  "providers.custom.model": "Model name",
  "providers.custom.modelHint":
    "As the service names it, for example openai/gpt-4o-mini on OpenRouter.",
  "providers.custom.policies":
    "Text sent to this service is handled under its own policies, and a gateway may pass it on to the company that runs the model.",
  "providers.custom.needsSetup":
    "Enter the service address, a model name, and an API key under Other AI service first.",
  "providers.custom.unnamed": "Other service",
  "providers.local.title": "Local AI",
  "providers.local.description":
    "Ollama runs language models on this computer. OpenTalk AI can use one for translation and reply suggestions, so that text never leaves the machine.",
  "providers.local.endpoint": "Ollama address",
  "providers.local.endpointHint":
    "Must be an address on this computer, normally http://localhost:11434.",
  "providers.local.check": "Check connection",
  "providers.local.checking": "Checking…",
  "providers.local.running": "Ollama {version} is running.",
  "providers.local.model": "Model",
  "providers.local.chooseModel": "Choose a model",
  "providers.local.noModels":
    'No models are installed. Install one yourself with Ollama, for example by running "ollama pull" followed by a model name in a terminal. OpenTalk AI never downloads models.',
  "providers.local.resources":
    "A model needs about as much free memory as its size, and each reply takes a few seconds. Quality depends on the model and varies by language.",
  "providers.local.error.not_local":
    "That address is not on this computer. Use an address such as http://localhost:11434.",
  "providers.local.error.not_running":
    "No Ollama server is answering at that address. Start Ollama and check again.",
  "providers.local.error.timeout": "Ollama did not answer in time.",
  "providers.local.error.malformed_response":
    "Something answered at that address, but it does not look like Ollama.",
  "providers.local.error.server": "Ollama reported an error.",
  "providers.local.error.unknown": "Local AI can be checked in the desktop app.",
  "providers.local.processing":
    "Handled by the model {model} in Ollama on this computer. Nothing is sent over the network.",
  "providers.local.needsModel": "Choose a local model under Local AI first.",
  "providers.translation.needsTranscription":
    "Translation needs transcription turned on, and a provider that is set up: a saved API key or a chosen local model.",
  "providers.replies.needsTranscription":
    "Reply suggestions need transcription turned on, and a provider that is set up: a saved API key or a chosen local model.",
  "providers.whisper.enable": "Transcribe captured speech with {provider} on this computer",
  "providers.whisper.title": "Local transcription",
  "providers.whisper.description":
    "Whisper is a speech recognition model that runs inside OpenTalk AI on this computer. It needs a model file and no other program.",
  "providers.whisper.processing":
    "Speech is turned into text on this computer by the Whisper model {model}. No audio is sent anywhere.",
  "providers.whisper.needsModel": "Choose a Whisper model under Local transcription first.",
  "providers.whisper.finalOnly":
    "Text appears when someone pauses, not while they are still speaking.",
  "providers.whisper.languages":
    "The models are multilingual, with accuracy that varies by language. Larger models are more accurate and slower.",
  "providers.whisper.model": "Model",
  "providers.whisper.chooseModel": "Choose a model",
  "providers.whisper.noModels":
    "No model is installed yet. Download one below, or put a Whisper model file (ggml-*.bin) into the models folder.",
  "providers.whisper.download": "Download models",
  "providers.whisper.downloadNote":
    "A model is downloaded only when you press Download, from the whisper.cpp model repository on Hugging Face, and is checked against its published checksum before use.",
  "providers.whisper.downloadButton": "Download {model} ({size})",
  "providers.whisper.installed": "{model} is installed",
  "providers.whisper.downloading": "Downloading {model}: {percent}%",
  "providers.whisper.cancel": "Cancel",
  "providers.whisper.folder": "Models folder: {folder}",
  "providers.whisper.license":
    "Whisper models are published by OpenAI under the MIT license. A model needs about as much memory as its file size.",
  "providers.whisper.error.local_only":
    "Downloading is turned off in local-only mode, because it contacts a download server. Turn local-only mode off to download, then turn it on again, or copy a model file into the models folder.",
  "providers.whisper.error.connection_failed": "The download server could not be reached.",
  "providers.whisper.error.cancelled": "The download was cancelled.",
  "providers.whisper.error.verification_failed":
    "The downloaded file did not match the published model and was discarded.",
  "providers.whisper.error.storage_failed": "The model could not be saved to disk.",
  "providers.whisper.error.busy": "Another download is already running.",
  "providers.whisper.error.unknown_model": "That model is not in the list.",
  "providers.whisper.error.unknown": "Models can be managed in the desktop app.",
  "providers.custom.service": "Service",
  "providers.custom.other": "Other address",
  "providers.localOnly.noTranscription":
    "With a Whisper model for transcription and a local model in Ollama for translation and replies, everything runs on this computer.",
  "providers.custom.noTranscription":
    "These services are used for translation and reply suggestions. For transcription, choose OpenAI or the built-in Whisper model.",
  "live.transcription.localOnly":
    "Local-only mode is on. Choose a Whisper model under Providers to transcribe on this computer.",
  "providers.replies.description": "Drafts replies for you to review.",

  "privacy.intro": "How OpenTalk AI is designed to handle your conversations.",
  "privacy.principle.capture":
    "Capture stays off until you press Start, and a visible indicator shows when it is on.",
  "privacy.principle.audio": "Raw audio is not saved.",
  "privacy.principle.cloud": "Nothing is sent to a cloud provider unless you choose that provider.",
  "privacy.principle.autosend": "Replies are never sent automatically.",
  "privacy.buildStatus":
    "Audio is captured only between Start and Stop. Speech is sent to a transcription provider only if you turn that on under Providers.",
  "privacy.history.title": "Session history",
  "privacy.history.toggle": "Save session history on this device",
  "privacy.history.unavailable":
    "History is off and cannot be turned on yet, because local history storage is not implemented.",
  "privacy.stored.title": "Stored on this device",
  "privacy.stored.body":
    "Only the settings on these screens (theme, languages, business context and glossary) are stored, in this app's local storage.",
  "privacy.reset": "Reset all settings",
  "privacy.resetDone": "Settings were reset.",

  "context.intro":
    "Background that a reply provider can use so suggestions fit your business. It is saved on this device and is not sent anywhere in this build.",
  "context.profile": "About your business",
  "context.profileHint":
    "Services, working hours, prices you are willing to quote. Up to {max} characters.",
  "context.glossary.title": "Glossary",
  "context.glossary.intro": "Terms that should always be translated in a specific way.",
  "context.glossary.empty": "No terms yet.",
  "context.glossary.term": "Term",
  "context.glossary.preferred": "Preferred translation",
  "context.glossary.entry": "Term {number}",
  "context.glossary.add": "Add term",
  "context.glossary.remove": "Remove term {number}",
  "context.glossary.limit": "The limit of {max} terms has been reached.",

  "appearance.theme": "Theme",
  "appearance.theme.system": "Match system",
  "appearance.theme.light": "Light",
  "appearance.theme.dark": "Dark",
  "appearance.language": "Interface language",

  "rtl.intro":
    "Checks that Arabic and other right-to-left text is laid out correctly. The Arabic below is fixed sample text, not provider output.",
  "rtl.toggle": "Preview the whole interface right-to-left",
  "rtl.samples": "Sample text",
  "rtl.sample.plain": "Without tashkeel",
  "rtl.sample.tashkeel": "With full tashkeel",
  "rtl.sample.mixed": "Arabic with Latin text and numbers",
  "rtl.sample.url": "Arabic with a URL",

  "about.description": "Open-source desktop assistant for real-time multilingual conversations.",
  "about.status":
    "Development build. Transcription, translation and reply suggestions have not yet been tested against the live service.",
  "about.core.loading": "Connecting to the application core…",
  "about.core.ready": "Application core: {name} {version}",
  "about.core.unavailable":
    "Application core unavailable. Start the app with the Tauri shell to connect.",
  "about.license": "License: MIT",
} as const;

export type MessageKey = keyof typeof en;

/** The shape every locale must satisfy: one string per message key. */
export type Messages = Record<MessageKey, string>;
