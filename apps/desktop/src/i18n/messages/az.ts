import type { Messages } from "./en";

/** Azerbaijani interface text. Keys must match `en.ts`. */
export const az: Messages = {
  "app.name": "OpenTalk AI",
  "app.skipToContent": "Əsas məzmuna keç",

  "common.copy": "Kopyala",
  "common.copied": "Kopyalandı",
  "common.copyFailed": "Kopyalamaq alınmadı",
  "common.demoTag": "Demo",

  "nav.label": "Əsas naviqasiya",
  "nav.settingsGroup": "Ayarlar",
  "nav.live": "Canlı iş sahəsi",
  "nav.languages": "Dillər",
  "nav.providers": "Provayderlər",
  "nav.privacy": "Məxfilik",
  "nav.context": "Biznes konteksti",
  "nav.appearance": "Görünüş",
  "nav.rtl": "RTL önbaxışı",
  "nav.about": "Haqqında",

  "live.subtitle": "Söhbəti tutun, öz dilinizdə oxuyun və cavab təklifləri alın.",
  "live.conversation.title": "Söhbət",
  "live.languages.automatic": "Avtomatik",
  "live.state.label": "Sessiyanın vəziyyəti",
  "live.state.idle": "Gözləmədə",
  "live.state.listening": "Dinləyir",
  "live.state.processing": "Emal edir",
  "live.state.error": "Xəta",
  "live.elapsed.label": "Keçən vaxt",
  "live.elapsed.notRunning": "işləmir",
  "live.audio.title": "Səs mənbələri",
  "live.audio.microphone": "Mikrofon",
  "live.audio.system": "Sistem səsini tut",
  "live.audio.start": "Başlat",
  "live.audio.starting": "Başlayır…",
  "live.audio.startingNote":
    "Səsin tutulması başlayır. Lokal modelin yüklənməsi bir neçə saniyə çəkə bilər.",
  "live.audio.stop": "Dayandır",
  "live.audio.useMicrophone": "Mikrofonu tut",
  "live.audio.defaultMicrophone": "Sistemin standart mikrofonu",
  "live.audio.capturing": "Səs tutulur",
  "live.audio.level.microphone": "Mikrofonun səviyyəsi",
  "live.audio.level.system": "Sistem səsinin səviyyəsi",
  "live.audio.consent":
    "Başlamazdan əvvəl söhbətdəki hər kəsin səsin tutulmasına razı olduğuna əmin olun.",
  "live.error.no_source_selected": "Ən azı bir səs mənbəyi seçin.",
  "live.error.invalid_microphone_selection": "Seçilmiş mikrofon düzgün deyil. Başqasını seçin.",
  "live.error.already_running": "Səs artıq tutulur.",
  "live.error.permission_denied.microphone":
    "Mikrofona giriş rədd edildi. Sistemin məxfilik ayarlarında OpenTalk AI-yə mikrofondan istifadəyə icazə verin və yenidən cəhd edin.",
  "live.error.permission_denied.system":
    "Sistem səsinə giriş rədd edildi. macOS-da Məxfilik və Təhlükəsizlik bölməsində Ekran və Sistem Səsinin Yazılması icazəsini verin, sonra tətbiqi yenidən açın.",
  "live.error.device_unavailable.microphone": "Mikrofon tapılmadı.",
  "live.error.device_unavailable.system": "Tutmaq üçün səs çıxışı tapılmadı.",
  "live.error.unsupported": "Bu səs mənbəyi bu sistemdə dəstəklənmir.",
  "live.error.interrupted": "Səsin tutulması gözlənilmədən dayandı. Ola bilsin, cihaz ayrılıb.",
  "live.error.backend": "Səsin tutulmasını başlatmaq alınmadı.",
  "live.error.unknown": "Səsin tutulmasında xəta baş verdi.",
  "live.audio.notStored":
    "Səs heç vaxt saxlanılmır. O, bu kompüterdən yalnız transkripsiya yandırılanda çıxır.",
  "live.transcript.empty":
    "Hələ transkript yoxdur. Səs tutulanda və transkripsiya yandırılanda nitq burada görünür.",
  "live.transcription.off": "Transkripsiya söndürülüb. Onu Provayderlər bölməsində yandırın.",
  "live.transcription.on":
    "Transkripsiya yandırılıb: tutulan mənbələrdəki nitq bulud xidməti olan {provider}-a göndərilir.",
  "live.transcription.stopped": "Transkripsiya dayandı. Səsin tutulması davam edir.",
  "live.error.missing_api_key":
    "Transkripsiya provayderi üçün API açarı saxlanılmayıb. Onu Provayderlər bölməsində əlavə edin.",
  "live.error.authentication_failed":
    "Transkripsiya provayderi API açarını rədd etdi. Açarı Provayderlər bölməsində yoxlayın.",
  "live.error.rate_limited":
    "Transkripsiya provayderi sorğuları məhdudlaşdırır. Bir az gözləyib yenidən cəhd edin.",
  "live.error.unsupported_language":
    "Transkripsiya provayderi seçilmiş danışıq dilini dəstəkləmir.",
  "live.error.connection_failed":
    "Transkripsiya provayderinə qoşulmaq alınmadı. İnternet bağlantınızı yoxlayın.",
  "live.error.connection_lost": "Transkripsiya provayderi ilə bağlantı kəsildi.",
  "live.error.timeout": "Transkripsiya provayderi vaxtında cavab vermədi.",
  "live.error.malformed_response": "Transkripsiya provayderi anlaşılmayan cavab göndərdi.",
  "live.error.provider": "Transkripsiya provayderi xəta bildirdi.",
  "live.translation.pending": "Tərcümə olunur…",
  "live.translation.failed": "Tərcümə olunmadı: {reason}",
  "live.translation.check": "Bu tərcüməni yoxlayın. Bunlar düzgün ötürülməmiş ola bilər: {items}",
  "live.reason.missing_api_key": "API açarı saxlanılmayıb.",
  "live.reason.authentication_failed": "provayder API açarını rədd etdi.",
  "live.reason.rate_limited": "provayder sorğuları məhdudlaşdırır və ya hesabın limiti bitib.",
  "live.reason.unsupported_language": "provayder bu dili dəstəkləmir.",
  "live.reason.connection_failed": "provayderə qoşulmaq alınmadı.",
  "live.reason.timeout": "provayder vaxtında cavab vermədi.",
  "live.reason.malformed_response": "provayderin cavabı anlaşılmadı.",
  "live.reason.refused": "provayder bu mətni tərcümə etməkdən imtina etdi.",
  "live.reason.overloaded": "nitq tərcümə oluna biləndən daha sürətli gəlir.",
  "live.reason.provider": "provayder xəta bildirdi.",
  "live.reason.unknown": "xəta baş verdi.",
  "live.translating.on":
    "{language} dilinə tərcümə yandırılıb: tamamlanmış transkript mətni {provider}-a göndərilir.",
  "live.translating.off": "Tərcümə söndürülüb.",
  "live.demo.toggle": "Demo məlumatı göstər",
  "live.demo.hint": "Tərtibatı nəzərdən keçirmək üçün panelləri sabit nümunə mətnlə doldurur.",
  "live.demo.banner":
    "Demo rejimi. Aşağıdakı mətn sabit nümunə məlumatdır. Heç bir səs tutulmur, mətnə çevrilmir və ya tərcümə olunmur.",
  "live.translation.title": "Tərcümə",
  "live.segment.partial": "Davam edir",
  "live.source.microphone": "Mikrofon",
  "live.source.system": "Sistem səsi",
  "live.replies.title": "Cavab təklifləri",
  "live.replies.refresh": "Təklifləri yenilə",
  "live.replies.empty":
    "Hələ təklif yoxdur. Cavab təklifləri Provayderlər bölməsində yandırılanda, qarşı tərəf danışdıqdan sonra görünür. Sizin əvəzinizə heç vaxt heç nə göndərilmir.",
  "live.replies.refreshUnavailable":
    "Yeniləmə səs tutulanda və cavab təklifləri yandırılanda əlçatandır.",
  "live.replies.pending": "Yeni təkliflər hazırlanır…",
  "live.replies.outdated": "Bu təkliflər söhbətin əvvəlki hissəsinə aiddir.",
  "live.replies.failed": "Təklifləri hazırlamaq alınmadı: {reason}",
  "live.replies.dismiss": "Rədd et",
  "live.replies.dismissLabel": "{style} cavabı rədd et",
  "live.replies.asksForMissingFacts":
    "Bu cavab sual verir, çünki ona lazım olan fakt, məsələn qiymət və ya tarix, söhbətdə və ya biznes kontekstinizdə yoxdur.",
  "live.replies.on":
    "Cavab təklifləri yandırılıb: son söhbət və biznes kontekstiniz {provider}-a göndərilir.",
  "live.replies.off": "Cavab təklifləri söndürülüb.",
  "live.arabic.generatedDiacritics":
    "Bu mətndəki hərəkələri süni intellekt əlavə edib və onlar səhv ola bilər. Etibar etməzdən əvvəl yoxlayın.",
  "live.replies.speak": "Səsləndir",
  "live.replies.speakLabel": "{style} cavabı səsləndir",
  "live.replies.noVoice": "Bu dil üçün sistemdə səs quraşdırılmayıb.",
  "live.translating.onLocal":
    "{language} dilinə tərcümə yandırılıb; onu bu kompüterdə {provider} edir.",
  "live.replies.onLocal": "Cavab təklifləri yandırılıb; onları bu kompüterdə {provider} yazır.",
  "live.error.local_only":
    "Yalnız lokal rejim yandırılıb, bu sessiya isə bulud xidmətindən istifadə edərdi. Provayderlər bölməsində provayderi dəyişin və ya yalnız lokal rejimi söndürün.",
  "live.reason.model_unavailable": "seçilmiş model əlçatan deyil.",
  "live.transcription.onLocal":
    "Transkripsiya yandırılıb; onu bu kompüterdə {provider} edir. Heç bir səs kompüterdən çıxmır.",
  "live.error.model_unavailable":
    "Seçilmiş model əlçatan deyil. Provayderlər bölməsində model seçin.",
  "live.replies.style.professional": "Peşəkar",
  "live.replies.style.friendly": "Səmimi",
  "live.replies.style.concise": "Qısa",
  "live.replies.meaning": "Mənası",
  "live.replies.edit": "Redaktə et",
  "live.replies.doneEditing": "Hazırdır",
  "live.replies.editLabel": "{style} cavabı redaktə et",

  "languages.intro":
    "Bu seçimlər bu cihazda saxlanılır. Hansı dillərin həqiqətən işləməsi provayderdən asılıdır; sınaqdan keçməyənə qədər heç biri dəstəklənən kimi göstərilmir.",
  "languages.source": "Danışıq dili",
  "languages.sourceAuto": "Avtomatik müəyyən et",
  "languages.target": "Tərcümə dili",
  "languages.reply": "Cavab təkliflərinin dili",
  "languages.tagHint": "Dil kodu, məsələn en, az, ru və ya ar-EG.",
  "languages.tagInvalid": "Düzgün dil kodu daxil edin, məsələn en, az, ru və ya ar-EG.",
  "languages.arabic.title": "Ərəbcə nəticə",
  "languages.arabic.scope":
    "Tərcümə və ya cavab dili ərəbcə olanda tətbiq olunur. Təşkil söndürüləndə hərəkələr, provayder əlavə etsə belə, həmişə silinir.",
  "languages.arabic.dialectUnavailable":
    "Dialekt üstünlüyünü qəbul edən provayder əlçatan olanda dialekt seçmək mümkündür.",
  "languages.arabic.dialectUnverified":
    "Dialekt nəticəsi yoxlanılmayıb və standart ərəbcə ilə qarışa bilər; vacib mətnləri ana dili daşıyıcısına yoxlatdırın.",
  "languages.arabic.variety": "Variant",
  "languages.arabic.variety.msa": "Müasir standart ərəb dili",
  "languages.arabic.variety.dialect": "Dialekt",
  "languages.arabic.dialectLabel": "Dialektin adı",
  "languages.arabic.dialectHint": "Məsələn, Misir və ya Körfəz dialekti.",
  "languages.arabic.tashkeel": "Təşkil (hərəkələr)",
  "languages.arabic.tashkeel.off": "Söndürülüb",
  "languages.arabic.tashkeel.partial": "Qismən",
  "languages.arabic.tashkeel.full": "Tam",
  "languages.arabic.tashkeelWarning":
    "Hərəkələri model yaradır və onlar səhv ola bilər. Etibar etməzdən əvvəl yoxlayın.",

  "providers.intro":
    "OpenTalk AI hər tapşırıq üçün ayrıca, əvəz edilə bilən provayderdən istifadə edir.",
  "providers.location.cloud": "Bulud xidməti",
  "providers.location.local": "Bu kompüterdə işləyir",
  "providers.disclosure":
    "Transkripsiya yandırılanda tutulan səsin nitq olan hissələri şifrələnmiş bağlantı ilə {provider}-a göndərilir və orada mətnə çevrilir. Səssizlik göndərilmir. {provider} bu səslə öz məlumat siyasətinə uyğun davranır.",
  "providers.capability.partial": "Kimsə hələ danışarkən mətni göstərir.",
  "providers.capability.noPartial": "Mətni yalnız danışan dayandıqdan sonra göstərir.",
  "providers.capability.languageDetection": "Aşkarladığı dili bildirir.",
  "providers.capability.noLanguageDetection": "Hansı dili aşkarladığını bildirmir.",
  "providers.capability.languagesUnknown":
    "Dəstəklənən dillərin siyahısını dərc etmir; dəstəklənməyən dil səsin tutulması başlayanda rədd edilir.",
  "providers.capability.languages": "Dəstəklənən dillər: {languages}",
  "providers.key.label": "API açarı",
  "providers.key.saved": "API açarı sistemin etimadnamə anbarında saxlanılıb.",
  "providers.key.missing": "API açarı saxlanılmayıb.",
  "providers.key.hint":
    "Açar əməliyyat sisteminin etimadnamə anbarında saxlanılır və bir daha göstərilmir.",
  "providers.key.save": "Açarı saxla",
  "providers.key.remove": "Açarı sil",
  "providers.key.error.invalid_key":
    "Bu açar düzgün deyil. O, boş olmamalı və boşluq saxlamamalıdır.",
  "providers.key.error.store_unavailable":
    "Sistemin etimadnamə anbarından istifadə etmək alınmadı.",
  "providers.enable": "Tutulan nitqi transkripsiya üçün {provider}-a göndər",
  "providers.enableNeedsKey": "Bunu yandırmaq üçün əvvəlcə API açarını saxlayın.",
  "providers.unavailable": "Transkripsiya ayarları masaüstü tətbiqdə əlçatandır.",
  "providers.transcription": "Transkripsiya",
  "providers.translation": "Tərcümə",
  "providers.translation.description": "Transkripti tərcümə edir.",
  "providers.translation.disclosure":
    "Tərcümə yandırılanda hər tamamlanmış transkript seqmentinin mətni və lüğətiniz şifrələnmiş bağlantı ilə {provider}-a göndərilir və orada tərcümə olunur. {provider}-dan onları saxlamamaq xahiş olunur.",
  "providers.translation.languagesUnknown":
    "Dəstəklənən dil cütlərinin siyahısını dərc etmir, ona görə burada heç biri iddia edilmir. İstifadə etdiyiniz dillər üçün nəticəni yoxlayın.",
  "providers.translation.checks":
    "Hər tərcümə yoxlanılır: orijinaldakı rəqəm, keçid və ya lüğət termini çatışmırsa, yoxlamağınız üçün işarələnir.",
  "providers.translation.enable": "Transkript mətnini tərcümə üçün {provider}-a göndər",
  "providers.replies": "Cavab təklifləri",
  "providers.replies.disclosure":
    "Cavab təklifləri yandırılanda son söhbət (ən çoxu son 24 replika) və biznes kontekstiniz şifrələnmiş bağlantı ilə {provider}-a göndərilir və cavabları o hazırlayır. {provider}-dan onları saxlamamaq xahiş olunur.",
  "providers.replies.rules":
    "Təkliflər özünüz oxuyub, redaktə edib, deməyiniz üçün qaralamalardır. OpenTalk AI sizin adınızdan heç vaxt heç nə göndərmir.",
  "providers.replies.source":
    "Təkliflər söhbətin qarşı tərəfinə cavab verir; onu sistem səsi tutur.",
  "providers.replies.enable": "Söhbəti cavab təklifləri üçün {provider}-a göndər",
  "providers.speech": "Səsləndirmə",
  "providers.speech.systemVoices": "Sistem səsləri",
  "providers.speech.description":
    "Sistem səsləri seçildikdə mətni bu kompüterdə quraşdırılmış səslər oxuyur. Heç nə heç yerə göndərilmir.",
  "providers.speech.voices":
    "Hansı dillərin səsləndirilə bilməsi sistem ayarlarında quraşdırılmış səslərdən asılıdır.",
  "providers.speech.enable": "Cavab təkliflərində Səsləndir düyməsini göstər",
  "providers.speech.unavailable": "Bu sistem tətbiqə səsləndirmə imkanı vermir.",
  "providers.speech.provider": "Səs",
  "providers.speech.provider.system": "Sistem səsləri, bu kompüterdə",
  "providers.speech.provider.elevenlabs": "ElevenLabs, bulud xidməti",
  "providers.speech.provider.compatible": "OpenAI tipli səsləndirmə API-si olan xidmət",
  "providers.speech.cloudDescription":
    "Səsləndirilən hər mətn şifrələnmiş bağlantı ilə {provider} bulud xidmətinə göndərilir və orada səsə çevrilir. Başqa heç nə göndərilmir.",
  "providers.speech.localDescription":
    "Səsləndirilən hər mətni bu kompüterdəki {provider} serveri səsə çevirir. Heç nə kompüterdən çıxmır.",
  "providers.speech.model": "Səsləndirmə modeli",
  "providers.speech.elevenlabs.voice": "Səsin ID-si",
  "providers.speech.elevenlabs.voiceHint": "ElevenLabs səs kitabxananızdakı səsin ID-si.",
  "providers.speech.elevenlabs.modelHint":
    "Hansı dillərin səsləndirilə biləcəyi modeldən asılıdır. eleven_v3 70-dən çox dildə, o cümlədən Azərbaycan dilində danışır.",
  "providers.speech.baseUrlHint":
    "API-nin əsas ünvanı, məsələn https://api.openai.com/v1. Xidmət bu kompüterdə işləmirsə, https ilə başlamalıdır.",
  "providers.speech.compatible.modelHint":
    "Xidmətin onu adlandırdığı kimi, məsələn OpenAI-də gpt-4o-mini-tts.",
  "providers.speech.compatible.voice": "Səs",
  "providers.speech.compatible.voiceHint": "Modelin səslərindən biri, məsələn alloy.",
  "providers.speech.translations.enable": "Qarşı tərəfin tərcümələrini səsləndir",
  "providers.speech.translations.description":
    "Səs tutulduğu müddətdə sistem səsindən gələn hər bitmiş tərcümə, deyiləndən bir neçə saniyə sonra, sizin dilinizdə səsləndirilir. Mikrofon bu səsi tutmasın deyə qulaqlıqdan istifadə edin.",
  "providers.speech.translations.needsTranslation":
    "Tərcümə söndürülüb, ona görə səsləndiriləcək heç nə yoxdur. Onu yuxarıda yandırın.",
  "providers.speech.translations.noVoice":
    "{language} üçün sistem səsi quraşdırılmayıb. Sistem ayarlarında quraşdırın və ya yuxarıda başqa səs seçin.",
  "live.speech.on": "Qarşı tərəfin tərcümələrini bu kompüterdə {provider} səsləndirir.",
  "live.speech.onCloud":
    "Qarşı tərəfin tərcümələri səsləndirilir: mətnləri {provider} bulud xidmətinə göndərilir.",
  "live.speech.needsTranslation":
    "Tərcümələrin səsləndirilməsi yandırılıb, amma tərcümə söndürülüb, ona görə heç nə səslənmir.",
  "live.speech.failed": "Tərcüməni səsləndirmək alınmadı: {reason}",
  "live.speech.reason.invalid_settings": "səs ayarları natamamdır və ya düzgün deyil.",
  "live.speech.reason.local_only": "yalnız lokal rejim yandırılıb.",
  "providers.localOnly.title": "Yalnız lokal rejim",
  "providers.localOnly.toggle": "Heç vaxt bulud xidməti ilə əlaqə saxlama",
  "providers.localOnly.description":
    "Bu yandırılanda bulud provayderlərini seçmək olmur və tətbiq hər hansı məlumatı onlara göndərəcək sessiyanı başlatmaqdan imtina edir.",
  "providers.localOnly.blocked": "Yalnız lokal rejim tərəfindən söndürülüb.",
  "providers.choice.label": "Provayder",
  "providers.choice.cloud": "{provider} · bulud xidməti",
  "providers.choice.local": "{provider} · bu kompüterdə",
  "providers.custom.title": "Digər süni intellekt xidməti",
  "providers.custom.description":
    "Tərcümə və cavab təklifləri üçün OpenAI-uyğun API təqdim edən istənilən xidmətdən, məsələn OpenRouter-dən istifadə edin. Onun ünvanını, model adını və həmin xidmət üçün API açarınızı daxil edin.",
  "providers.custom.baseUrl": "Xidmətin ünvanı",
  "providers.custom.baseUrlHint":
    "API-nin əsas URL-i, məsələn https://openrouter.ai/api/v1. Xidmət bu kompüterdə işləmirsə, https ilə başlamalıdır.",
  "providers.custom.baseUrlInvalid": "https ünvanı və ya bu kompüterdəki http ünvanı daxil edin.",
  "providers.custom.model": "Model adı",
  "providers.custom.modelHint":
    "Xidmətin adlandırdığı kimi, məsələn OpenRouter-də openai/gpt-4o-mini.",
  "providers.custom.policies":
    "Bu xidmətə göndərilən mətnlə onun öz siyasətinə uyğun davranılır və vasitəçi xidmət onu modeli işlədən şirkətə ötürə bilər.",
  "providers.custom.needsSetup":
    "Əvvəlcə Digər süni intellekt xidməti bölməsində xidmətin ünvanını, model adını və API açarını daxil edin.",
  "providers.custom.unnamed": "Digər xidmət",
  "providers.local.title": "Lokal süni intellekt",
  "providers.local.description":
    "Ollama dil modellərini bu kompüterdə işlədir. OpenTalk AI tərcümə və cavab təklifləri üçün ondan istifadə edə bilər; bu halda mətn kompüterdən çıxmır.",
  "providers.local.endpoint": "Ollama ünvanı",
  "providers.local.endpointHint": "Bu kompüterdəki ünvan olmalıdır, adətən http://localhost:11434.",
  "providers.local.check": "Bağlantını yoxla",
  "providers.local.checking": "Yoxlanılır…",
  "providers.local.running": "Ollama {version} işləyir.",
  "providers.local.model": "Model",
  "providers.local.chooseModel": "Model seçin",
  "providers.local.noModels":
    'Heç bir model quraşdırılmayıb. Modeli Ollama ilə özünüz quraşdırın, məsələn terminalda "ollama pull" və model adını yazaraq. OpenTalk AI heç vaxt model yükləmir.',
  "providers.local.resources":
    "Modelə təxminən öz ölçüsü qədər boş yaddaş lazımdır və hər cavab bir neçə saniyə çəkir. Keyfiyyət modeldən asılıdır və dilə görə dəyişir.",
  "providers.local.error.not_local":
    "Bu ünvan bu kompüterdə deyil. http://localhost:11434 kimi bir ünvandan istifadə edin.",
  "providers.local.error.not_running":
    "Bu ünvanda heç bir Ollama serveri cavab vermir. Ollama-nı işə salıb yenidən yoxlayın.",
  "providers.local.error.timeout": "Ollama vaxtında cavab vermədi.",
  "providers.local.error.malformed_response":
    "Bu ünvanda nəsə cavab verdi, amma o, Ollama-ya oxşamır.",
  "providers.local.error.server": "Ollama xəta bildirdi.",
  "providers.local.error.unknown": "Lokal süni intellekti masaüstü tətbiqdə yoxlamaq olar.",
  "providers.local.processing":
    "Bunu bu kompüterdə Ollama-dakı {model} modeli edir. Şəbəkə ilə heç nə göndərilmir.",
  "providers.local.needsModel": "Əvvəlcə Lokal süni intellekt bölməsində lokal model seçin.",
  "providers.translation.needsTranscription":
    "Tərcümə üçün transkripsiya yandırılmalı və provayder hazır olmalıdır: saxlanılmış API açarı və ya seçilmiş lokal model.",
  "providers.replies.needsTranscription":
    "Cavab təklifləri üçün transkripsiya yandırılmalı və provayder hazır olmalıdır: saxlanılmış API açarı və ya seçilmiş lokal model.",
  "providers.whisper.enable": "Tutulan nitqi bu kompüterdə {provider} ilə mətnə çevir",
  "providers.whisper.title": "Lokal transkripsiya",
  "providers.whisper.description":
    "Whisper OpenTalk AI-nin içində, bu kompüterdə işləyən nitq tanıma modelidir. Ona yalnız model faylı lazımdır, başqa proqram tələb olunmur.",
  "providers.whisper.processing":
    "Nitq bu kompüterdə {model} Whisper modeli ilə mətnə çevrilir. Heç bir səs heç yerə göndərilmir.",
  "providers.whisper.needsModel": "Əvvəlcə Lokal transkripsiya bölməsində Whisper modeli seçin.",
  "providers.whisper.finalOnly": "Mətn danışan hələ danışarkən yox, fasilə verəndə görünür.",
  "providers.whisper.languages":
    "Modellər çoxdillidir, dəqiqlik isə dilə görə dəyişir. Böyük modellər daha dəqiq və daha yavaşdır.",
  "providers.whisper.model": "Model",
  "providers.whisper.chooseModel": "Model seçin",
  "providers.whisper.noModels":
    "Hələ heç bir model quraşdırılmayıb. Aşağıdan birini yükləyin və ya Whisper model faylını (ggml-*.bin) modellər qovluğuna qoyun.",
  "providers.whisper.download": "Modelləri yüklə",
  "providers.whisper.downloadNote":
    "Model yalnız siz Yüklə düyməsini basanda, Hugging Face-dəki whisper.cpp model anbarından yüklənir və istifadədən əvvəl dərc olunmuş yoxlama cəmi ilə tutuşdurulur.",
  "providers.whisper.downloadButton": "{model} modelini yüklə ({size})",
  "providers.whisper.installed": "{model} quraşdırılıb",
  "providers.whisper.downloading": "{model} yüklənir: {percent}%",
  "providers.whisper.cancel": "Ləğv et",
  "providers.whisper.folder": "Modellər qovluğu: {folder}",
  "providers.whisper.license":
    "Whisper modellərini OpenAI MIT lisenziyası ilə dərc edib. Modelə təxminən fayl ölçüsü qədər yaddaş lazımdır.",
  "providers.whisper.error.local_only":
    "Yalnız lokal rejimdə yükləmə söndürülüb, çünki o, yükləmə serveri ilə əlaqə saxlayır. Yükləmək üçün yalnız lokal rejimi söndürün, sonra yenidən yandırın, və ya model faylını modellər qovluğuna köçürün.",
  "providers.whisper.error.connection_failed": "Yükləmə serverinə qoşulmaq alınmadı.",
  "providers.whisper.error.cancelled": "Yükləmə ləğv edildi.",
  "providers.whisper.error.verification_failed":
    "Yüklənmiş fayl dərc olunmuş modelə uyğun gəlmədi və silindi.",
  "providers.whisper.error.storage_failed": "Modeli diskə saxlamaq alınmadı.",
  "providers.whisper.error.busy": "Başqa bir yükləmə artıq davam edir.",
  "providers.whisper.error.unknown_model": "Bu model siyahıda yoxdur.",
  "providers.whisper.error.unknown": "Modelləri masaüstü tətbiqdə idarə etmək olar.",
  "providers.custom.service": "Xidmət",
  "providers.custom.other": "Başqa ünvan",
  "providers.localOnly.noTranscription":
    "Transkripsiya üçün Whisper modeli, tərcümə və cavablar üçün Ollama-da lokal model olanda hər şey bu kompüterdə işləyir.",
  "providers.custom.noTranscription":
    "Bu xidmətlər tərcümə və cavab təklifləri üçün istifadə olunur. Transkripsiya üçün OpenAI-ni və ya quraşdırılmış Whisper modelini seçin.",
  "live.transcription.localOnly":
    "Yalnız lokal rejim yandırılıb. Bu kompüterdə transkripsiya üçün Provayderlər bölməsində Whisper modeli seçin.",
  "providers.replies.description": "Nəzərdən keçirməyiniz üçün cavablar hazırlayır.",

  "privacy.intro": "OpenTalk AI söhbətlərinizlə necə davranmaq üçün nəzərdə tutulub.",
  "privacy.principle.capture":
    "Siz Başlat düyməsini basana qədər səs tutulmur, tutulanda isə bu, aydın göstərilir.",
  "privacy.principle.audio": "Xam səs saxlanılmır.",
  "privacy.principle.cloud":
    "Özünüz seçmədiyiniz halda heç bir bulud provayderinə heç nə göndərilmir.",
  "privacy.principle.autosend": "Cavablar heç vaxt avtomatik göndərilmir.",
  "privacy.buildStatus":
    "Səs yalnız Başlat və Dayandır arasında tutulur. Nitq transkripsiya provayderinə yalnız siz bunu Provayderlər bölməsində yandırsanız göndərilir.",
  "privacy.history.title": "Sessiya tarixçəsi",
  "privacy.history.toggle": "Sessiya tarixçəsini bu cihazda saxla",
  "privacy.history.unavailable":
    "Tarixçə söndürülüb və hələ yandırıla bilməz, çünki lokal tarixçə anbarı hazır deyil.",
  "privacy.stored.title": "Bu cihazda saxlanılanlar",
  "privacy.stored.body":
    "Yalnız bu ekranlardakı ayarlar (mövzu, dillər, biznes konteksti və lüğət) bu tətbiqin lokal yaddaşında saxlanılır.",
  "privacy.reset": "Bütün ayarları sıfırla",
  "privacy.resetDone": "Ayarlar sıfırlandı.",

  "context.intro":
    "Təkliflərin biznesinizə uyğun olması üçün cavab provayderinin istifadə edə biləcəyi məlumat. Bu cihazda saxlanılır və bu versiyada heç yerə göndərilmir.",
  "context.profile": "Biznesiniz haqqında",
  "context.profileHint":
    "Xidmətlər, iş saatları, təklif etməyə hazır olduğunuz qiymətlər. Ən çoxu {max} simvol.",
  "context.glossary.title": "Lüğət",
  "context.glossary.intro": "Həmişə müəyyən şəkildə tərcümə olunmalı terminlər.",
  "context.glossary.empty": "Hələ termin yoxdur.",
  "context.glossary.term": "Termin",
  "context.glossary.preferred": "Üstün tutulan tərcümə",
  "context.glossary.entry": "Termin {number}",
  "context.glossary.add": "Termin əlavə et",
  "context.glossary.remove": "{number}-ci termini sil",
  "context.glossary.limit": "{max} terminlik limitə çatılıb.",

  "appearance.theme": "Mövzu",
  "appearance.theme.system": "Sistemə uyğun",
  "appearance.theme.light": "Açıq",
  "appearance.theme.dark": "Tünd",
  "appearance.language": "İnterfeys dili",

  "rtl.intro":
    "Ərəb və digər sağdan sola yazılan mətnlərin düzgün düzüldüyünü yoxlayır. Aşağıdakı ərəbcə mətn provayderin nəticəsi deyil, sabit nümunədir.",
  "rtl.toggle": "Bütün interfeysi sağdan sola göstər",
  "rtl.samples": "Nümunə mətn",
  "rtl.sample.plain": "Təşkilsiz",
  "rtl.sample.tashkeel": "Tam təşkillə",
  "rtl.sample.mixed": "Latın mətni və rəqəmlərlə ərəbcə",
  "rtl.sample.url": "URL ilə ərəbcə",

  "about.description": "Real vaxtda çoxdilli söhbətlər üçün açıq mənbəli masaüstü köməkçi.",
  "about.status":
    "İnkişaf versiyası. Transkripsiya, tərcümə və cavab təklifləri hələ canlı xidmətlə sınaqdan keçirilməyib.",
  "about.core.loading": "Tətbiqin nüvəsinə qoşulur…",
  "about.core.ready": "Tətbiqin nüvəsi: {name} {version}",
  "about.core.unavailable":
    "Tətbiqin nüvəsi əlçatan deyil. Qoşulmaq üçün tətbiqi Tauri ilə işə salın.",
  "about.license": "Lisenziya: MIT",
};
