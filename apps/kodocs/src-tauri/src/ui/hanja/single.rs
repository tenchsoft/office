use super::HanjaEntry;

/// Single-character Korean to Hanja mapping for common characters.
pub(super) fn single_char_hanja(ch: char) -> Vec<HanjaEntry> {
    match ch {
        '가' => vec![
            HanjaEntry {
                hanja: "家",
                meaning: "집 가",
            },
            HanjaEntry {
                hanja: "歌",
                meaning: "노래 가",
            },
            HanjaEntry {
                hanja: "佳",
                meaning: "아름다울 가",
            },
        ],
        '나' => vec![
            HanjaEntry {
                hanja: "我",
                meaning: "나 나",
            },
            HanjaEntry {
                hanja: "羅",
                meaning: "그물 나",
            },
        ],
        '다' => vec![
            HanjaEntry {
                hanja: "多",
                meaning: "많을 다",
            },
            HanjaEntry {
                hanja: "茶",
                meaning: "차 다",
            },
        ],
        '사' => vec![
            HanjaEntry {
                hanja: "事",
                meaning: "일 사",
            },
            HanjaEntry {
                hanja: "四",
                meaning: "넉 사",
            },
            HanjaEntry {
                hanja: "思",
                meaning: "생각 사",
            },
            HanjaEntry {
                hanja: "社",
                meaning: "모일 사",
            },
            HanjaEntry {
                hanja: "士",
                meaning: "선비 사",
            },
        ],
        '명' => vec![
            HanjaEntry {
                hanja: "名",
                meaning: "이름 명",
            },
            HanjaEntry {
                hanja: "明",
                meaning: "밝을 명",
            },
            HanjaEntry {
                hanja: "命",
                meaning: "목숨 명",
            },
            HanjaEntry {
                hanja: "銘",
                meaning: "새길 명",
            },
        ],
        '성' => vec![
            HanjaEntry {
                hanja: "性",
                meaning: "성품 성",
            },
            HanjaEntry {
                hanja: "姓",
                meaning: "성씨 성",
            },
            HanjaEntry {
                hanja: "成",
                meaning: "이룰 성",
            },
            HanjaEntry {
                hanja: "聲",
                meaning: "소리 성",
            },
            HanjaEntry {
                hanja: "城",
                meaning: "성 성",
            },
        ],
        '년' => vec![
            HanjaEntry {
                hanja: "年",
                meaning: "해 년",
            },
            HanjaEntry {
                hanja: "然",
                meaning: "그러할 연",
            },
        ],
        '월' => vec![
            HanjaEntry {
                hanja: "月",
                meaning: "달 월",
            },
            HanjaEntry {
                hanja: "越",
                meaning: "넘을 월",
            },
        ],
        '일' => vec![
            HanjaEntry {
                hanja: "日",
                meaning: "날 일",
            },
            HanjaEntry {
                hanja: "一",
                meaning: "하나 일",
            },
            HanjaEntry {
                hanja: "日",
                meaning: "해 일",
            },
            HanjaEntry {
                hanja: "業",
                meaning: "업 일",
            },
        ],
        '학' => vec![
            HanjaEntry {
                hanja: "學",
                meaning: "배울 학",
            },
            HanjaEntry {
                hanja: "鶴",
                meaning: "학 학",
            },
        ],
        '생' => vec![
            HanjaEntry {
                hanja: "生",
                meaning: "날 생",
            },
            HanjaEntry {
                hanja: "聲",
                meaning: "소리 성",
            },
        ],
        '문' => vec![
            HanjaEntry {
                hanja: "文",
                meaning: "글월 문",
            },
            HanjaEntry {
                hanja: "門",
                meaning: "문 문",
            },
            HanjaEntry {
                hanja: "問",
                meaning: "물을 문",
            },
        ],
        '수' => vec![
            HanjaEntry {
                hanja: "水",
                meaning: "물 수",
            },
            HanjaEntry {
                hanja: "數",
                meaning: "셈 수",
            },
            HanjaEntry {
                hanja: "手",
                meaning: "손 수",
            },
            HanjaEntry {
                hanja: "守",
                meaning: "지킬 수",
            },
            HanjaEntry {
                hanja: "樹",
                meaning: "나무 수",
            },
        ],
        '대' => vec![
            HanjaEntry {
                hanja: "大",
                meaning: "큰 대",
            },
            HanjaEntry {
                hanja: "代",
                meaning: "대신 대",
            },
            HanjaEntry {
                hanja: "對",
                meaning: "대답 대",
            },
            HanjaEntry {
                hanja: "帶",
                meaning: "띠 대",
            },
        ],
        '동' => vec![
            HanjaEntry {
                hanja: "東",
                meaning: "동녁 동",
            },
            HanjaEntry {
                hanja: "動",
                meaning: "움직일 동",
            },
            HanjaEntry {
                hanja: "同",
                meaning: "같을 동",
            },
            HanjaEntry {
                hanja: "童",
                meaning: "아이 동",
            },
        ],
        '서' => vec![
            HanjaEntry {
                hanja: "書",
                meaning: "글 서",
            },
            HanjaEntry {
                hanja: "西",
                meaning: "서녁 서",
            },
            HanjaEntry {
                hanja: "序",
                meaning: "차례 서",
            },
            HanjaEntry {
                hanja: "暑",
                meaning: "더울 서",
            },
        ],
        '남' => vec![
            HanjaEntry {
                hanja: "南",
                meaning: "남녁 남",
            },
            HanjaEntry {
                hanja: "男",
                meaning: "사내 남",
            },
            HanjaEntry {
                hanja: "納",
                meaning: "바칠 납",
            },
        ],
        '북' => vec![HanjaEntry {
            hanja: "北",
            meaning: "북녁 북",
        }],
        '중' => vec![
            HanjaEntry {
                hanja: "中",
                meaning: "가운데 중",
            },
            HanjaEntry {
                hanja: "重",
                meaning: "무거울 중",
            },
            HanjaEntry {
                hanja: "衆",
                meaning: "무리 중",
            },
        ],
        '국' => vec![
            HanjaEntry {
                hanja: "國",
                meaning: "나라 국",
            },
            HanjaEntry {
                hanja: "菊",
                meaning: "국화 국",
            },
            HanjaEntry {
                hanja: "局",
                meaning: "국 국",
            },
        ],
        '강' => vec![
            HanjaEntry {
                hanja: "江",
                meaning: "강 강",
            },
            HanjaEntry {
                hanja: "强",
                meaning: "강할 강",
            },
            HanjaEntry {
                hanja: "康",
                meaning: "편안할 강",
            },
        ],
        '산' => vec![
            HanjaEntry {
                hanja: "山",
                meaning: "뫼 산",
            },
            HanjaEntry {
                hanja: "算",
                meaning: "셈 산",
            },
        ],
        '시' => vec![
            HanjaEntry {
                hanja: "時",
                meaning: "때 시",
            },
            HanjaEntry {
                hanja: "市",
                meaning: "시장 시",
            },
            HanjaEntry {
                hanja: "詩",
                meaning: "시 시",
            },
            HanjaEntry {
                hanja: "始",
                meaning: "비로소 시",
            },
            HanjaEntry {
                hanja: "視",
                meaning: "볼 시",
            },
        ],
        '인' => vec![
            HanjaEntry {
                hanja: "人",
                meaning: "사람 인",
            },
            HanjaEntry {
                hanja: "仁",
                meaning: "어질 인",
            },
            HanjaEntry {
                hanja: "印",
                meaning: "도장 인",
            },
            HanjaEntry {
                hanja: "因",
                meaning: "인할 인",
            },
        ],
        '정' => vec![
            HanjaEntry {
                hanja: "正",
                meaning: "바를 정",
            },
            HanjaEntry {
                hanja: "政",
                meaning: "정사 정",
            },
            HanjaEntry {
                hanja: "情",
                meaning: "뜻 정",
            },
            HanjaEntry {
                hanja: "定",
                meaning: "정할 정",
            },
            HanjaEntry {
                hanja: "靜",
                meaning: "고요할 정",
            },
        ],
        '경' => vec![
            HanjaEntry {
                hanja: "京",
                meaning: "서울 경",
            },
            HanjaEntry {
                hanja: "景",
                meaning: "경치 경",
            },
            HanjaEntry {
                hanja: "敬",
                meaning: "공경할 경",
            },
            HanjaEntry {
                hanja: "經",
                meaning: "지날 경",
            },
            HanjaEntry {
                hanja: "鏡",
                meaning: "거울 경",
            },
        ],
        '도' => vec![
            HanjaEntry {
                hanja: "道",
                meaning: "길 도",
            },
            HanjaEntry {
                hanja: "都",
                meaning: "도읍 도",
            },
            HanjaEntry {
                hanja: "圖",
                meaning: "그림 도",
            },
            HanjaEntry {
                hanja: "度",
                meaning: "법도 도",
            },
        ],
        '자' => vec![
            HanjaEntry {
                hanja: "字",
                meaning: "글자 자",
            },
            HanjaEntry {
                hanja: "子",
                meaning: "아들 자",
            },
            HanjaEntry {
                hanja: "自",
                meaning: "스스로 자",
            },
            HanjaEntry {
                hanja: "者",
                meaning: "놈 자",
            },
        ],
        '세' => vec![
            HanjaEntry {
                hanja: "世",
                meaning: "세상 세",
            },
            HanjaEntry {
                hanja: "歲",
                meaning: "해 세",
            },
            HanjaEntry {
                hanja: "勢",
                meaning: "세력 세",
            },
        ],
        '개' => vec![
            HanjaEntry {
                hanja: "個",
                meaning: "낱 개",
            },
            HanjaEntry {
                hanja: "開",
                meaning: "열 개",
            },
            HanjaEntry {
                hanja: "改",
                meaning: "고칠 개",
            },
        ],
        '신' => vec![
            HanjaEntry {
                hanja: "新",
                meaning: "새 신",
            },
            HanjaEntry {
                hanja: "神",
                meaning: "귀신 신",
            },
            HanjaEntry {
                hanja: "身",
                meaning: "몸 신",
            },
            HanjaEntry {
                hanja: "信",
                meaning: "믿을 신",
            },
        ],
        '의' => vec![
            HanjaEntry {
                hanja: "義",
                meaning: "옳을 의",
            },
            HanjaEntry {
                hanja: "意",
                meaning: "뜻 의",
            },
            HanjaEntry {
                hanja: "醫",
                meaning: "의원 의",
            },
            HanjaEntry {
                hanja: "衣",
                meaning: "옷 의",
            },
        ],
        '력' => vec![
            HanjaEntry {
                hanja: "力",
                meaning: "힘 력",
            },
            HanjaEntry {
                hanja: "歷",
                meaning: "지날 력",
            },
            HanjaEntry {
                hanja: "曆",
                meaning: "책력 력",
            },
        ],
        '공' => vec![
            HanjaEntry {
                hanja: "工",
                meaning: "장인 공",
            },
            HanjaEntry {
                hanja: "公",
                meaning: "공변될 공",
            },
            HanjaEntry {
                hanja: "功",
                meaning: "공 공",
            },
            HanjaEntry {
                hanja: "空",
                meaning: "빌 공",
            },
        ],
        '기' => vec![
            HanjaEntry {
                hanja: "機",
                meaning: "틀 기",
            },
            HanjaEntry {
                hanja: "氣",
                meaning: "기운 기",
            },
            HanjaEntry {
                hanja: "期",
                meaning: "기약 기",
            },
            HanjaEntry {
                hanja: "記",
                meaning: "기록할 기",
            },
            HanjaEntry {
                hanja: "技",
                meaning: "재주 기",
            },
            HanjaEntry {
                hanja: "起",
                meaning: "일어날 기",
            },
        ],
        '전' => vec![
            HanjaEntry {
                hanja: "前",
                meaning: "앞 전",
            },
            HanjaEntry {
                hanja: "全",
                meaning: "온전할 전",
            },
            HanjaEntry {
                hanja: "電",
                meaning: "번개 전",
            },
            HanjaEntry {
                hanja: "戰",
                meaning: "싸울 전",
            },
            HanjaEntry {
                hanja: "傳",
                meaning: "전할 전",
            },
        ],
        '안' => vec![
            HanjaEntry {
                hanja: "安",
                meaning: "편안할 안",
            },
            HanjaEntry {
                hanja: "案",
                meaning: "책상 안",
            },
        ],
        '상' => vec![
            HanjaEntry {
                hanja: "上",
                meaning: "위 상",
            },
            HanjaEntry {
                hanja: "相",
                meaning: "서로 상",
            },
            HanjaEntry {
                hanja: "商",
                meaning: "장사 상",
            },
            HanjaEntry {
                hanja: "想",
                meaning: "생각 상",
            },
        ],
        '하' => vec![
            HanjaEntry {
                hanja: "下",
                meaning: "아래 하",
            },
            HanjaEntry {
                hanja: "河",
                meaning: "물 하",
            },
            HanjaEntry {
                hanja: "夏",
                meaning: "여름 하",
            },
        ],
        '주' => vec![
            HanjaEntry {
                hanja: "主",
                meaning: "임금 주",
            },
            HanjaEntry {
                hanja: "住",
                meaning: "살 주",
            },
            HanjaEntry {
                hanja: "酒",
                meaning: "술 주",
            },
            HanjaEntry {
                hanja: "注",
                meaning: "부을 주",
            },
        ],
        _ => Vec::new(),
    }
}
