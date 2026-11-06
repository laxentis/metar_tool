# METAR Tool — instrukcja obsługi

Instrukcja dotyczy wersji 1.0.0. Nazwy pól i przycisków podano tak, jak występują w angielskim interfejsie programu.

## 1. Do czego służy program

METAR Tool jest aplikacją do ręcznego przygotowywania depesz METAR i SPECI. Na podstawie danych wpisanych w formularzu tworzy tekst depeszy, który można skopiować do schowka lub wysłać do skonfigurowanych serwerów. Program udostępnia również przypomnienia dźwiękowe o pełnych godzinach.

Dane obserwacyjne wprowadza użytkownik. Program nie pobiera aktualnej pogody ani nie aktualizuje automatycznie dnia i czasu obserwacji. Przed każdą wysyłką sprawdź dane oraz gotowy tekst depeszy. Formularz nie zapewnia pełnej kontroli poprawności meteorologicznej.

## 2. Uruchomienie i zamknięcie

Jeśli masz gotową wersję dla systemu Windows, uruchom plik `metar_tool.exe`.

Jeśli korzystasz z kodu źródłowego, potrzebujesz środowiska Rust z narzędziem Cargo. W katalogu projektu wykonaj:

```sh
cargo run
```

Aby zbudować wersję zoptymalizowaną, wykonaj:

```sh
cargo build --release
```

Przy standardowej konfiguracji Cargo plik wynikowy znajdziesz w `target/release/metar_tool.exe`.

Do przygotowania i kopiowania depeszy nie jest wymagane połączenie z siecią. Wysyłanie wymaga dostępu do wskazanych serwerów, a przypomnienia — działającego urządzenia audio.

Program zamkniesz przyciskiem zamknięcia okna lub poleceniem **File → Quit**. Aplikacja zapisuje stan i ustawienia oraz odtwarza je przy kolejnym uruchomieniu. Przywrócone dane obserwacji mogą być nieaktualne.

<img width="602" height="532" alt="Interfejs" src="https://github.com/user-attachments/assets/641c96be-0330-4c22-8a5c-e18bea5f69cc" />

## 3. Przygotowanie depeszy

### Typ, stacja i termin obserwacji

1. Z listy u góry okna wybierz **METAR**, **METAR COR**, **SPECI** lub **SPECI COR**. Warianty z `COR` służą do przygotowania depeszy korygującej.
2. W polu **ICAO** wpisz kod stacji, np. `EPBC`. Sprawdź zapis wielkimi literami.
3. W polu **Day** ustaw dzień miesiąca od 1 do 31.
4. W polu **Time** ustaw godzinę i minutę obserwacji w UTC. Program dopisuje do grupy czasu literę `Z`; nie przelicza czasu lokalnego na UTC.

Przykład: dzień `6` i czas `12:30` tworzą grupę `061230Z`.

### Wiatr — Wind

| Pole | Sposób użycia |
| --- | --- |
| Kierunek w stopniach | Ustaw kierunek w zakresie dostępnym w formularzu: 0–350°. W depeszy program zaokrągla go do dziesiątek stopni. |
| **VRB** | Zaznacz, aby zamiast kierunku liczbowego użyć oznaczenia `VRB`. |
| **V** | Zaznacz, aby włączyć przedział zmienności kierunku. Ustaw jego granice suwakiem, np. 200° i 340°. |
| **Speed** | Wprowadź prędkość w węzłach (`kts`), od 0 do 100. W depeszy jednostkę oznacza `KT`. |
| **G** | Zaznacz, aby dodać porywy, i wpisz ich prędkość w sąsiednim polu. |

Przykładowy wynik: `28010G20KT 200V340`.

### Widzialność — Visibility

Ustaw widzialność suwakiem **Visibility** w metrach, od 0 do 10 000.

Program formatuje wartości od 5000 do 9998 m do pełnych tysięcy metrów w dół, np. 7500 m zapisuje jako `7000`. Wartość co najmniej 9999 m zapisuje jako `9999`, o ile nie zostanie użyte `CAVOK`.

Przy 10 000 m i pustych listach chmur oraz zjawisk program może automatycznie wyświetlić `CAVOK`. Przy braku warstw chmur, gdy nie zastosowano `CAVOK`, dodaje `NSC`. Sprawdź wynik przed wysłaniem; szczególne ograniczenie automatycznego `CAVOK` opisano w rozdziale 9.

### Chmury — Clouds

<img width="602" height="532" alt="Cloud edit" src="https://github.com/user-attachments/assets/3d51cb68-1dd2-4608-8f27-05a8245bb2b3" />

1. Przy **Clouds** kliknij przycisk z plusem.
2. W oknie **Edit cloud layer** wybierz pokrycie nieba: `FEW`, `SCT`, `BKN` lub `OVC`.
3. W polu **Height** wpisz wysokość podstawy w stopach (`ft`), np. `2000`. W depeszy zostanie ona zapisana w setkach stóp, np. `BKN020`.
4. Wybierz typ chmur: **None** — bez dodatkowego oznaczenia, **CB** lub **TCU**.
5. Zamknij okno edycji. Zmiany są uwzględniane na bieżąco.

Powtórz te czynności dla kolejnych warstw. Przycisk edycji obok warstwy otwiera jej ustawienia, a przycisk kosza usuwa ją z listy. Warstwy pojawiają się w depeszy w kolejności dodawania; dodawaj je w wymaganej kolejności.

### Zjawiska — Phenomenon

<img width="602" height="532" alt="Edit phenomenon" src="https://github.com/user-attachments/assets/e2d03c0f-860c-46be-8c57-1236bc41942f" />

1. Przy **Phenomenon** kliknij przycisk z plusem.
2. W oknie **Edit phenomenon** wybierz natężenie: **None** — bez znaku, **+** albo **−**.
3. Wybierz kod zjawiska z listy, np. `RA`, `SN`, `DZ`, `BR` lub `FG`.
4. W razie potrzeby wybierz modyfikator, np. `SH`, `TS` lub `FZ`. Wybierz **None**, aby pozostawić zjawisko bez modyfikatora.
5. Zamknij okno edycji.

Przykład: natężenie `−`, kod `RA` i modyfikator `SH` dają `-SHRA`.

Możesz dodać kilka pozycji. Do zmiany lub usunięcia pozycji służą przyciski obok jej kodu. Formularz umożliwia wybranie jednego modyfikatora na pozycję.

### Temperatura, punkt rosy i QNH

| Pole | Wprowadzana wartość | Zakres formularza |
| --- | --- | --- |
| **Temperature** | Temperatura powietrza w °C | od −50 do 50 |
| **Dew Point** | Temperatura punktu rosy w °C | od −50 do 50 |
| **QNH** | Ciśnienie w hPa | od 800 do 1200 |

QNH jest zapisywane bez części ułamkowej, np. `1013,25` daje `Q1013`. Przy temperaturach ujemnych sprawdź zapis w depeszy — ograniczenie obecnej wersji opisano w rozdziale 9.

## 4. Podgląd, kopiowanie i ręczna edycja

Tekst depeszy jest widoczny na dole okna i aktualizuje się po zmianach w formularzu. Program dodaje na końcu znak `=`.

Kliknij przycisk z ikoną schowka, aby skopiować depeszę utworzoną z formularza.

Aby ręcznie zmienić tekst:

1. Zaznacz **Overwrite**.
2. Wprowadź poprawki w aktywnym polu tekstowym obok tej opcji.
3. Sprawdź cały tekst, w tym stację, termin i znak końcowy.

**Przy włączonym Overwrite przycisk Send to servers wysyła tekst z pola ręcznej edycji. Przycisk schowka nadal kopiuje tekst utworzony z formularza.** Aby skopiować ręcznie zmienioną depeszę, zaznacz tekst w polu edycji i użyj `Ctrl+C`.

Podgląd nad polem ręcznej edycji nadal pokazuje wynik formularza. Zmiany w formularzu przy aktywnym **Overwrite** nie aktualizują ręcznie edytowanego tekstu. Wyłączenie tej opcji powoduje ponowne zastąpienie zawartości pola wynikiem formularza.

## 5. Konfiguracja serwerów

<img width="602" height="532" alt="Server settings" src="https://github.com/user-attachments/assets/645ce7bd-0f08-4908-9ddd-ca1560ef696a" />

### Dodawanie serwera

1. Otwórz **Settings → Servers**.
2. Kliknij **Add server**.
3. W polu **Name** wpisz nazwę rozpoznawalną dla użytkownika, np. `Serwer główny`.
4. Wpisz pełny adres odbiorczy zaczynający się od `https://` lub `http://`.
5. Jeśli serwer wymaga klucza, wpisz go w **Optional API key**. W przeciwnym razie pozostaw pole puste.

Klucz jest wysyłany w nagłówku `X-API-Key`. Możesz dodać kilka serwerów; każdy otrzyma tę samą depeszę. **Remove** usuwa wybraną pozycję. Przy pustej nazwie komunikaty używają oznaczenia `Server 1`, `Server 2` itd. Pozycje z pustym adresem są pomijane przy wysyłaniu.

### Wczytywanie konfiguracji z pliku

W oknie **Server settings** wpisz ścieżkę do pliku JSON, a następnie kliknij **Load config**. Najłatwiej użyć pełnej ścieżki, np. `C:\METAR\servers.json`. Ścieżki względne są rozpatrywane względem katalogu roboczego aplikacji.

Przykładowa zawartość pliku:

```json
{
  "servers": [
    {
      "name": "Serwer główny",
      "url": "https://serwer.example/metar",
      "api_key": "TUTAJ_WPISZ_KLUCZ"
    },
    {
      "name": "Serwer lokalny",
      "url": "http://localhost:8080/metar"
    }
  ]
}
```

Adresy są przykładowe — zastąp je właściwymi. Każda pozycja wymaga pola `url`; pola `name` i `api_key` są opcjonalne.

Poprawne wczytanie **zastępuje całą bieżącą listę serwerów**. Błąd odczytu lub niepoprawna konfiguracja pozostawiają dotychczasową listę. Plik z zawartością `{"servers": []}` usuwa wszystkie skonfigurowane serwery.

### Eksport konfiguracji

W tym samym polu ścieżki wpisz docelową lokalizację i kliknij **Export config**. Program zapisze bieżącą listę w formacie JSON.

**Eksport zawiera klucze API w jawnym tekście i nadpisuje istniejący plik pod wskazaną ścieżką.** Przechowuj taki plik w miejscu dostępnym wyłącznie dla uprawnionych osób.

## 6. Wysyłanie depeszy

1. Sprawdź depeszę oraz listę serwerów.
2. Ustal, czy chcesz wysłać wynik formularza, czy tekst ręczny — zależy to od stanu **Overwrite**.
3. Kliknij **Send to servers**.
4. Poczekaj na wynik dla każdego serwera.

Program wysyła tekst żądaniem HTTP POST, kolejno do wszystkich skonfigurowanych adresów. Limit czasu pojedynczego żądania wynosi 15 sekund, więc przy kilku niedostępnych serwerach oczekiwanie może być dłuższe. Wyniki są wyświetlane osobno dla każdego serwera; powodzenie jednej wysyłki nie oznacza powodzenia pozostałych.

| Komunikat lub status | Co zrobić |
| --- | --- |
| `accepted`, HTTP 200 | Sprawdź szczegóły odpowiedzi, w tym stację, termin oraz ewentualne zastąpienie wcześniejszej depeszy. Jeśli podano `ok=false`, wyjaśnij wynik z administratorem serwera. |
| HTTP 200 z informacją o niepoprawnym JSON | Serwer odpowiedział, ale aplikacja nie mogła odczytać szczegółów. Zweryfikuj zapis po stronie serwera. |
| HTTP 400 | Depesza została odrzucona. Przeczytaj przyczynę podaną w komunikacie, popraw tekst i wyślij ponownie. |
| HTTP 401 | Sprawdź klucz API dla tego serwera. |
| HTTP 404 | Sprawdź pełny adres punktu odbiorczego. |
| HTTP 5xx | Wystąpił błąd serwera. Spróbuj ponownie później. |
| `no response` | Sprawdź połączenie, dostępność serwera i jego adres. |
| Prośba o dodanie adresu serwera | Uzupełnij konfigurację w **Settings → Servers**. |
| `METAR is empty; nothing was sent.` | Uzupełnij ręcznie edytowany tekst lub wyłącz **Overwrite**. |

Program nie ponawia wysyłki automatycznie. Kolejne kliknięcie **Send to servers** wysyła depeszę ponownie do wszystkich pozycji z niepustym adresem.

## 7. Przypomnienia dźwiękowe

<img width="602" height="532" alt="Sound settings" src="https://github.com/user-attachments/assets/b80afe31-ab98-4a35-8467-b95d13be2bff" />

1. Otwórz **Settings → Sound**.
2. Zaznacz **Enabled**, aby włączyć przypomnienia, lub odznacz, aby je wyłączyć.
3. Suwakiem **Active hours** ustaw godziny aktywności.
4. Kliknij **Test sound**, aby sprawdzić dźwięk.

Przypomnienia korzystają z **lokalnego czasu komputera**. Przy ustawieniu 7–19 sygnał może zabrzmieć o pełnych godzinach od 07:00 do 19:00 włącznie. Domyślnie przypomnienia są włączone dla tego przedziału.

Aplikacja musi działać, aby odtworzyć przypomnienie. Sygnał nie wysyła depeszy. Gdy nie słychać dźwięku, sprawdź głośność i domyślne urządzenie wyjściowe; jeśli urządzenie nie było dostępne podczas uruchamiania, podłącz je i uruchom program ponownie.

## 8. Przykład przygotowania depeszy

Wprowadź następujące dane:

| Ustawienie | Wartość |
| --- | --- |
| Typ | METAR |
| ICAO | EPBC |
| Day / Time | 6 / 12:00 UTC |
| Wiatr | 280°, 5 kts; VRB, V i G wyłączone |
| Visibility | 5000 m |
| Clouds | Jedna warstwa: FEW, 1200 ft, typ None |
| Phenomenon | Jedna pozycja: DZ, natężenie None, modyfikator None |
| Temperature / Dew Point | 15°C / 5°C |
| QNH | 1013 hPa |

Usuń inne warstwy i zjawiska, jeśli pozostały z poprzedniej obserwacji. Wynik formularza powinien wyglądać tak:

```text
METAR EPBC 061200Z 28005KT 5000 DZ FEW012 15/05 Q1013=
```

Aby wysłać ten wynik, wyłącz **Overwrite** i kliknij **Send to servers** po skonfigurowaniu odbiorców.

## 9. Ograniczenia obecnej wersji

Poniższe uwagi wynikają z działania wersji 1.0.0 i są istotne przy sprawdzaniu gotowej depeszy:

- **Temperatury ujemne:** generator używa znaku minus zamiast oznaczenia `M`. W razie potrzeby popraw grupę temperatur w trybie **Overwrite** przed wysłaniem.
- **Automatyczne CAVOK:** przy widzialności 10 000 m, jeśli lista chmur nie została jeszcze utworzona, program może wyświetlić `CAVOK` również po dodaniu zjawiska pogodowego, pomijając je w wyniku. Sprawdź grupę widzialności i zjawisk; w razie potrzeby popraw tekst w **Overwrite**.
- **Kopiowanie ręcznych poprawek:** ikona schowka zawsze kopiuje wynik formularza. Tekst z **Overwrite** kopiuj bezpośrednio z pola edycji.
- **Dodatkowe grupy:** formularz nie udostępnia osobnych pól m.in. dla RVR, widzialności kierunkowej, AUTO ani TREND. Wymagane uzupełnienia można wpisać ręcznie w **Overwrite**.
- **Przypomnienia po ponownym uruchomieniu:** zapisane ustawienia dźwięku są odtwarzane w oknie, ale usługa przypomnień startuje z wartościami domyślnymi. Aby przekazać jej ustawienia widoczne w oknie, zmień **Enabled** lub **Active hours**; możesz np. wyłączyć i ponownie włączyć **Enabled**.

## 10. Kontrola przed wysłaniem

1. Sprawdź typ depeszy, kod ICAO oraz dzień i czas UTC.
2. Zweryfikuj wiatr, widzialność, zjawiska, warstwy chmur, temperatury i QNH.
3. Przeczytaj gotowy tekst, zwłaszcza jeśli występują temperatury ujemne lub automatyczne `CAVOK`.
4. Sprawdź stan **Overwrite** oraz zawartość pola, które zostanie wysłane.
5. Po wysyłce sprawdź wynik dla każdego odbiorcy.
