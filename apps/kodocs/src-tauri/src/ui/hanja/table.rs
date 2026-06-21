use super::HanjaEntry;

/// Common Korean word-to-Hanja mappings.
pub(super) static HANJA_TABLE: &[(&str, &[HanjaEntry])] = &[
    (
        "학교",
        &[HanjaEntry {
            hanja: "學校",
            meaning: "배울 곳",
        }],
    ),
    (
        "학생",
        &[HanjaEntry {
            hanja: "學生",
            meaning: "배우는 사람",
        }],
    ),
    (
        "선생",
        &[HanjaEntry {
            hanja: "先生",
            meaning: "먼저 난 사람",
        }],
    ),
    (
        "시간",
        &[HanjaEntry {
            hanja: "時間",
            meaning: "때의 간",
        }],
    ),
    (
        "세계",
        &[HanjaEntry {
            hanja: "世界",
            meaning: "세상의 경계",
        }],
    ),
    (
        "국가",
        &[HanjaEntry {
            hanja: "國家",
            meaning: "나라와 집",
        }],
    ),
    (
        "사람",
        &[HanjaEntry {
            hanja: "사람",
            meaning: "인간 (純韓語)",
        }],
    ),
    (
        "문제",
        &[HanjaEntry {
            hanja: "問題",
            meaning: "물을 제목",
        }],
    ),
    (
        "경제",
        &[HanjaEntry {
            hanja: "經濟",
            meaning: "다스릴 경, 건널 제",
        }],
    ),
    (
        "정치",
        &[HanjaEntry {
            hanja: "政治",
            meaning: "바를 정, 다스릴 치",
        }],
    ),
    (
        "사회",
        &[HanjaEntry {
            hanja: "社會",
            meaning: "모일 사, 모일 회",
        }],
    ),
    (
        "문화",
        &[HanjaEntry {
            hanja: "文化",
            meaning: "글월 문, 변할 화",
        }],
    ),
    (
        "과학",
        &[HanjaEntry {
            hanja: "科學",
            meaning: "과목 과, 배울 학",
        }],
    ),
    (
        "기술",
        &[HanjaEntry {
            hanja: "技術",
            meaning: "재주 기, 꾀 술",
        }],
    ),
    (
        "자연",
        &[HanjaEntry {
            hanja: "自然",
            meaning: "스스로 자, 그러할 연",
        }],
    ),
    (
        "환경",
        &[HanjaEntry {
            hanja: "環境",
            meaning: "고리 환, 지경 경",
        }],
    ),
    (
        "교육",
        &[HanjaEntry {
            hanja: "敎育",
            meaning: "가르칠 교, 기를 육",
        }],
    ),
    (
        "역사",
        &[HanjaEntry {
            hanja: "歷史",
            meaning: "지날 력, 기록 사",
        }],
    ),
    (
        "가족",
        &[HanjaEntry {
            hanja: "家族",
            meaning: "집 가, 겨레 족",
        }],
    ),
    (
        "친구",
        &[HanjaEntry {
            hanja: "親舊",
            meaning: "친할 친, 옛 구",
        }],
    ),
    (
        "음악",
        &[HanjaEntry {
            hanja: "音樂",
            meaning: "소리 음, 즐길 악",
        }],
    ),
    (
        "운동",
        &[HanjaEntry {
            hanja: "運動",
            meaning: "운전할 운, 움직일 동",
        }],
    ),
    (
        "건강",
        &[HanjaEntry {
            hanja: "健康",
            meaning: "건실할 건, 편안할 강",
        }],
    ),
    (
        "의학",
        &[HanjaEntry {
            hanja: "醫學",
            meaning: "의원 의, 배울 학",
        }],
    ),
    (
        "법률",
        &[HanjaEntry {
            hanja: "法律",
            meaning: "법 법, 법 률",
        }],
    ),
    (
        "민주",
        &[HanjaEntry {
            hanja: "民主",
            meaning: "백성 민, 임금 주",
        }],
    ),
    (
        "자유",
        &[HanjaEntry {
            hanja: "自由",
            meaning: "스스로 자, 말미할 유",
        }],
    ),
    (
        "평화",
        &[HanjaEntry {
            hanja: "平和",
            meaning: "평평할 평, 고를 화",
        }],
    ),
    (
        "미래",
        &[HanjaEntry {
            hanja: "未來",
            meaning: "아닐 미, 올 래",
        }],
    ),
    (
        "현재",
        &[HanjaEntry {
            hanja: "現在",
            meaning: "나타날 현, 있을 재",
        }],
    ),
    (
        "공간",
        &[HanjaEntry {
            hanja: "空間",
            meaning: "빌 공, 사이 간",
        }],
    ),
    (
        "인생",
        &[HanjaEntry {
            hanja: "人生",
            meaning: "사람 인, 날 생",
        }],
    ),
    (
        "사랑",
        &[HanjaEntry {
            hanja: "사랑",
            meaning: "사랑 (純韓語)",
        }],
    ),
    (
        "행복",
        &[HanjaEntry {
            hanja: "幸福",
            meaning: "행운 행, 복 복",
        }],
    ),
    (
        "노력",
        &[HanjaEntry {
            hanja: "努力",
            meaning: "힘 력",
        }],
    ),
    (
        "성공",
        &[HanjaEntry {
            hanja: "成功",
            meaning: "이룰 성, 공 공",
        }],
    ),
    (
        "발전",
        &[HanjaEntry {
            hanja: "發展",
            meaning: "쏠 발, 펼칠 전",
        }],
    ),
    (
        "기회",
        &[HanjaEntry {
            hanja: "機會",
            meaning: "틀 기, 모일 회",
        }],
    ),
    (
        "준비",
        &[HanjaEntry {
            hanja: "準備",
            meaning: "수평 준, 갖출 비",
        }],
    ),
    (
        "결과",
        &[HanjaEntry {
            hanja: "結果",
            meaning: "맺을 결, 과실 과",
        }],
    ),
    (
        "원인",
        &[HanjaEntry {
            hanja: "原因",
            meaning: "근원 원, 인할 인",
        }],
    ),
    (
        "방법",
        &[HanjaEntry {
            hanja: "方法",
            meaning: "모퉁이 방, 법 법",
        }],
    ),
    (
        "목적",
        &[HanjaEntry {
            hanja: "目的",
            meaning: "눈 목, 적일 적",
        }],
    ),
    (
        "의미",
        &[HanjaEntry {
            hanja: "意味",
            meaning: "뜻 의, 맛 미",
        }],
    ),
    (
        "가능",
        &[HanjaEntry {
            hanja: "可能",
            meaning: "할 가, 능할 능",
        }],
    ),
    (
        "변화",
        &[HanjaEntry {
            hanja: "變化",
            meaning: "변할 변, 변할 화",
        }],
    ),
    (
        "발명",
        &[HanjaEntry {
            hanja: "發明",
            meaning: "쏠 발, 밝을 명",
        }],
    ),
    (
        "전통",
        &[HanjaEntry {
            hanja: "傳統",
            meaning: "전할 전, 실마리 통",
        }],
    ),
    (
        "현대",
        &[HanjaEntry {
            hanja: "現代",
            meaning: "나타날 현, 대신 대",
        }],
    ),
    (
        "도덕",
        &[HanjaEntry {
            hanja: "道德",
            meaning: "길 도, 덕 덕",
        }],
    ),
    (
        "책임",
        &[HanjaEntry {
            hanja: "責任",
            meaning: "책망할 책, 맡길 임",
        }],
    ),
    (
        "권리",
        &[HanjaEntry {
            hanja: "權利",
            meaning: "저울추 권, 날카로울 리",
        }],
    ),
    (
        "의무",
        &[HanjaEntry {
            hanja: "義務",
            meaning: "옳을 의, 힘쓸 무",
        }],
    ),
];
