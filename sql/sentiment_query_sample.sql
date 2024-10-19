create table if not exists tree_select_ds(
    id int generated always as identity,
    threshold int,
    query_tree treearena
);

insert into
    tree_select_ds (threshold, query_tree)
VALUES
    (
        12,
        '{1{2 Something}{0{1{2 has}{1{2 been}{0{2 lost}{1{2 in}{0{2{2{2 the}{2 translation}}{2 ...}}{1{1{2 another}{2{2 routine}{1{2 Hollywood}{3 frightfest}}}}{1{2{2 in}{2 which}}{1{2{2 the}{1{2 slack}{2 execution}}}{2{2 italicizes}{1{1{2 the}{1 absurdity}}{2{2 of}{2{2 the}{2 premise}}}}}}}}}}}}}{2 .}}}'
    ),
    (
        9,
        '{4{2{2{2 A}{2 coda}}{2{2 in}{2{2{2{2 every}{2 sense}}{2 ,}}{2{2 The}{2{1 Pinochet}{2 Case}}}}}}{2{2{2 splits}{2{2 time}{3{2 between}{2{2{2 a}{2{2 minute-by-minute}{2 account}}}{2{2 of}{2{2{2{2{2 the}{2{2 British}{2{2 court}{2 ''s}}}}{2{2 extradition}{2{2 chess}{2 game}}}}{2 and}}{3{2{2 the}{2{2 regime}{2 ''s}}}{2{2 talking-head}{2 survivors}}}}}}}}}{2 .}}}'
    ),
    (
        11,
        '{3{2{2 The}{1{2 dirty}{2 jokes}}}{3{4{2 provide}{3{3{2 the}{3{4 funniest}{2 moments}}}{3{2 in}{3{3{2 this}{3{3{2 oddly}{4 sweet}}{3 comedy}}}{2{2 about}{3{2 jokester}{2{2 highway}{2 patrolmen}}}}}}}}{2 .}}}'
    ),
    (
        11,
        '{2{2 Ah}{2{2 ,}{2{2{2{2 the}{2 travails}}{2{2 of}{2{2 metropolitan}{3 life}}}}{2 !}}}}'
    ),
    (
        9,
        '{1{3{2 A}{3{4 worthy}{2 idea}}}{1{2{2{2 ,}{2 but}}{1{1{2 the}{1{1 uninspired}{2{2 scripts}{2{2 ,}{2{2{2 acting}{2 and}}{2 direction}}}}}}{1{2 never}{1{2 rise}{1{2 above}{1{2{2 the}{2 level}}{2{2 of}{2{2 an}{2{2 after-school}{2{3 TV}{3 special}}}}}}}}}}}{2 .}}}'
    ),
    (
        10,
        '{3{2{2{3 Chris}{2{2 Columbus}{2 ''}}}{2 sequel}}{4{3{2 is}{4{3{3{2{2 faster}{2 ,}}{2 livelier}}{2 and}}{4{3{2 a}{3{3 good}{3{2 deal}{3 funnier}}}}{2{2 than}{2{2 his}{3 original}}}}}}{2 .}}}'
    ),
    (
        10,
        '{1{2 `}{0{0{0{2{2{1{2 God}{2{2 help}{3 us}}}{2 ,}}{2 but}}{1{2{2{2 Capra}{2 and}}{2 Cooper}}{0{2 are}{0{1{2 rolling}{2 over}}{2{2 in}{2{2 their}{2 graves}}}}}}}{2 .}}{2 ''}}}'
    ),
    (
        9,
        '{0{2 Rarely}{0{0{2 does}{0{1{2{2 a}{2 film}}{0{2 so}{1{1{1 graceless}{2 and}}{2 devoid}}}}{2{2 of}{2{2 merit}{2{2 as}{2{2{2 this}{2 one}}{2{2 come}{2 along}}}}}}}}{2 .}}}'
    ),
    (
        11,
        '{2{2{2 It}{2{2{2 may}{2 even}}{2{2 fall}{2{2 into}{2{2{2 the}{2 category}}{2{2 of}{2 Films}}}}}}}{2{2 You}{2{2{4 Love}{1{2 to}{1 Hate}}}{2 .}}}}'
    ),
    (
        9,
        '{3{2{2 As}{2{2{2{2 ex-Marine}{2 Walter}}{2 ,}}{2{2 who}{2{2{2{2{2 may}{2 or}}{2 may}}{1 not}}{2{2 have}{2{2 shot}{2 Kennedy}}}}}}}{4{2 ,}{4{2{2 actor}{2{2 Raymond}{2{2 J.}{2 Barry}}}}{3{3{2 is}{3{4 perfectly}{3{2{1 creepy}{2 and}}{3 believable}}}}{2 .}}}}}'
    ),
    (
        9,
        '{0{0{1{1 Pompous}{2 and}}{2 garbled}}{2 .}}'
    ),
    (
        12,
        '{1{2 And}{1{2{2 in}{2{2 a}{2 sense}}}{1{2 ,}{1{2 that}{2{1{2 ''s}{0{2 a}{1 liability}}}{2 .}}}}}}'
    ),
    (
        10,
        '{3{3{3{2 The}{2{4 ingenious}{2 construction}}}{2{1 -LRB-}{2{2{2 adapted}{2{2 by}{2{2{2 David}{2 Hare}}{2{2 from}{2{2{2 Michael}{2{2 Cunningham}{2 ''s}}}{3 novel}}}}}}{3 -RRB-}}}}{3{2 constantly}{3{2{2{2 flows}{2{2{2 forwards}{2 and}}{2 back}}}{2 ,}}{3{2{2 weaving}{2 themes}}{3{2 among}{2{2{2 three}{2 strands}}{2{2 which}{2{2{2 allow}{3 us}}{2{2{2{2 to}{2{2 view}{2 events}}}{2{2 as}{2 if}}}{1{2 through}{2{2 a}{2 prism}}}}}}}}}}}}'
    ),
    (
        12,
        '{3{3{4{2 A}{4{3{3 solidly}{2 constructed}}{4{2 ,}{3{4 entertaining}{3 thriller}}}}}{2{2 that}{2{2 stops}{3{2 short}{4{2 of}{3{3 true}{4 inspiration}}}}}}}{2 .}}'
    ),
    (
        9,
        '{1{2{1{0{2 A}{1 waste}}{2{2 of}{3{3 fearless}{2 purity}}}}{2{2 in}{2{2 the}{2{2 acting}{2 craft}}}}}{2 .}}'
    ),
    (
        10,
        '{3{2{1 Bleakly}{3 funny}}{3{2 ,}{3{2{2{2 its}{2 characters}}{2{2 all}{2{2 the}{2 more}}}}{3{3{3 touching}{2{2 for}{2{2 refusing}{2{2 to}{2{2{1{1 pity}{2 or}}{2 memorialize}}{2 themselves}}}}}}{2 .}}}}}'
    ),
    (
        12,
        '{3{3{2{2{2 The}{2{2 series}{2 ''}}}{2 message}}{2{2 about}{3{3{2 making}{3{2 the}{3{3 right}{2 choice}}}}{2{2 in}{2{2{2 the}{2 face}}{2{2 of}{2{2 tempting}{2 alternatives}}}}}}}}{3{3{3{3{1 remains}{3 prominent}}{2 ,}}{3{2 as}{2{2 do}{3{2{2 the}{2{2 girls}{2 ''}}}{3{4 amusing}{2 personalities}}}}}}{2 .}}}'
    ),
    (
        12,
        '{3{3{2{2 An}{2{2 edifying}{2 glimpse}}}{2{2 into}{2{3{2 the}{2{2 wit}{3{2 and}{3{3 revolutionary}{3 spirit}}}}}{2{2 of}{2{2{2{2 these}{2 performers}}{2 and}}{2{2 their}{2 era}}}}}}}{2 .}}'
    ),
    (
        9,
        '{2{2{2 A}{1 puzzling}}{2{2 experience}{2 .}}}'
    ),
    (
        12,
        '{2{2{2 In}{2{2 Moonlight}{2 Mile}}}{2{2 ,}{2{2{1 no}{2 one}}{2{2{2 gets}{2{1{2 shut}{1 out}}{2{2 of}{2{2 the}{2{2 hug}{2 cycle}}}}}}{2 .}}}}}'
    ),
    (
        11,
        '{0{1{2{2 Mr.}{2{2 Goyer}{2 ''s}}}{0{1{1 loose}{2{2 ,}{2 unaccountable}}}{2 direction}}}{1{0{2 is}{1{3 technically}{1{3 sophisticated}{1{2 in}{2{2 the}{1{0 worst}{2 way}}}}}}}{2 .}}}'
    ),
    (
        11,
        '{3{2 `}{3{2 What}{3{2 ''s}{3{2{2 the}{2{2 Russian}{2 word}}}{2{2 for}{2{4 Wow}{2{2 !?}{2 ''}}}}}}}}'
    ),
    (
        11,
        '{1{2{2 As}{2{2{2 underwater}{2{2 ghost}{2 stories}}}{2 go}}}{3{2 ,}{3{2 Below}{2{2{2{2{3{2{2{2{2{3{2 casts}{2{2 its}{2{2 spooky}{2 net}}}}{1 out}}{2{2 into}{2{2 the}{2{2 Atlantic}{2 Ocean}}}}}{2 and}}{2{2{2 spits}{2 it}}{2 back}}}{2 ,}}{2 grizzled}}{2 and}}{1{2{2{2 charred}{2 ,}}{2{2 somewhere}{2 northwest}}}{2{2 of}{2{2 the}{2{2 Bermuda}{2 Triangle}}}}}}{2 .}}}}}'
    ),
    (
        9,
        '{3{3{2{1{1{2 You}{1{2{2 would}{2 n''t}}{2{2 want}{2{2 to}{2{2 live}{2 waydowntown}}}}}}{2 ,}}{2 but}}{3{2 it}{3{2 is}{3{2 a}{3{4 hilarious}{2{2 place}{2{2 to}{2 visit}}}}}}}}{2 .}}'
    ),
    (
        9,
        '{2{2{2 Meant}{2{2 for}{2{3 Star}{2{2 Wars}{3 fans}}}}}{2 .}}'
    ),
    (
        10,
        '{4{3 Compellingly}{3{3 watchable}{2 .}}}'
    ),
    (
        9,
        '{4{3 Refreshing}{2 .}}'
    ),
    (
        12,
        '{1{1{3{2 The}{2{2 punch}{2 lines}}}{1{2 that}{1 miss}}}{2{2 ,}{1{1 unfortunately}{2{2 ,}{1{2{2{2 outnumber}{2{2 the}{2 hits}}}{2{2 by}{2 three-to-one}}}{2 .}}}}}}'
    ),
    (
        9,
        '{3{4{4{3{3{2{2 While}{1{1{2 somewhat}{2 less}}{2{2 than}{1{2 it}{2{2 might}{2{2 have}{2 been}}}}}}}{3{2 ,}{4{2{2 the}{2 film}}{3{2 is}{3{2 a}{4{3 good}{2 one}}}}}}}{2 ,}}{2 and}}{3{2 you}{3{2 ''ve}{3{2 got}{3{2 to}{3{3{2{2 hand}{2 it}}{2{2 to}{2{2 director}{2{2 George}{2 Clooney}}}}}{2{2 for}{2{2{2{2 biting}{1 off}}{3{2 such}{2{2 a}{2{2 big}{2 job}}}}}{2{2{2 the}{2{2 first}{2 time}}}{1 out}}}}}}}}}}{2 .}}'
    ),
    (
        11,
        '{2{2 It}{2{2{2{2{3{2 ''s}{3{2 all}{2{4 entertaining}{2 enough}}}}{2 ,}}{2 but}}{1{2{2 do}{2 n''t}}{1{2 look}{1{2 for}{2{2{2 any}{3{2 hefty}{1{2 anti-establishment}{2 message}}}}{2{2 in}{1{2 what}{3{2{2 is}{2 essentially}}{1{4{2{2 a}{2 whip-crack}}{3{2 of}{3{2 a}{2{2 buddy}{2 movie}}}}}{1{2 that}{1{2 ends}{2{2 with}{1{2 a}{1 whimper}}}}}}}}}}}}}}{2 .}}}'
    ),
    (
        10,
        '{0{2 There}{0{0{1{2 ''s}{1{1 nothing}{3{3 interesting}{2{2 in}{2 Unfaithful}}}}}{2 whatsoever}}{2 .}}}'
    ),
    (
        9,
        '{4{3{4{2 A}{4{3 lively}{3{2 and}{3{4 engaging}{2 examination}}}}}{2{2 of}{2{2 how}{1{2{2 similar}{2 obsessions}}{2{2 can}{1{2 dominate}{2{2 a}{3 family}}}}}}}}{2 .}}'
    ),
    (
        12,
        '{2{2{2 ``}{2{2 Punch-Drunk}{4{4 Love}{2 ''}}}}{3{3{2 is}{3{2 a}{2{2{2 little}{2{2 like}{2{2 a}{2{2 chocolate}{2 milk}}}}}{1 moustache}}}}{2 ...}}}'
    ),
    (
        11,
        '{1{2{2 Murder}{2{2 by}{2 Numbers}}}{1{2 just}{1{1{1{2 does}{2 n''t}}{2{3 add}{2 up}}}{2 .}}}}'
    ),
    (
        12,
        '{2{2 So-so}{3{3 entertainment}{2 .}}}'
    ),
    (
        12,
        '{3{2 Finally}{3{2 ,}{3{2{2{2{2{2 the}{2 French-produced}}{2 ``}}{1{2 Read}{2{2 My}{2 Lips}}}}{2 ''}}{2{3{2 is}{3{2{2 a}{2 movie}}{2{2 that}{3{2 understands}{2{2 characters}{2{2 must}{2{2 come}{2 first}}}}}}}}{2 .}}}}}'
    ),
    (
        11,
        '{0{2{2 The}{2 catch}}{1{1{2 is}{1{2 that}{1{2 they}{1{2 ''re}{0{2 stuck}{1{2 with}{1{2{2 a}{2 script}}{1{2 that}{1{2{1 prevents}{2 them}}{3{2 from}{4{2 firing}{3{2 on}{2{2 all}{2 cylinders}}}}}}}}}}}}}}{2 .}}}'
    ),
    (
        9,
        '{1{2{2 Serving}{2 Sara}}{0{0{2 should}{0{2 be}{1{2{3 served}{1{2 an}{2{3 eviction}{2 notice}}}}{2{2 at}{1{2{2 every}{3 theater}}{2{2 stuck}{3{2 with}{2 it}}}}}}}}{2 .}}}'
    ),
    (
        11,
        '{1{2 It}{2{2{2{2{2 ''s}{1 not}}{2{2{2 the}{1 least}}{1{2 of}{1{2 Afghan}{1 tragedies}}}}}{1{2 that}{1{2{2 this}{2{3 noble}{2 warlord}}}{1{2 would}{1{2 be}{1{2 consigned}{1{2 to}{2{2{2 the}{2 dustbin}}{2{2 of}{2 history}}}}}}}}}}{2 .}}}'
    ),
    (
        11,
        '{1{2{2{2 The}{2 events}}{2{2 of}{2{2 the}{2 film}}}}{3{2{2{2 are}{2{2 just}{2 so}}}{3{2 WEIRD}{2{2 that}{3{2 I}{2{3 honestly}{1{2 never}{0{2 knew}{1{2 what}{2{1{2 the}{0 hell}}{2{2 was}{2{2 coming}{2 next}}}}}}}}}}}}{2 .}}}'
    ),
    (
        9,
        '{3{2 ...}{3{2 there}{3{3{2 is}{3{3{2 enough}{3{3{3 originality}{2{2{2{2 in}{2 `}}{3 Life}}{2 ''}}}{3{2 to}{3{2{2 distance}{2 it}}{2{2 from}{2{2{2 the}{2 pack}}{3{2 of}{2{2 paint-by-number}{2{2 romantic}{2 comedies}}}}}}}}}}{3{2 that}{3{2{2 so}{2 often}}{2{2{2 end}{2 up}}{2{2 on}{2{2 cinema}{2 screens}}}}}}}}{2 .}}}}'
    ),
    (
        12,
        '{3{3{2{1 -LRB-}{2{2 A}{3 -RRB-}}}{3{4 satisfying}{2 niblet}}}{2 .}}'
    ),
    (
        11,
        '{2{2{2{3{2 The}{3{3 good}{2 thing}}}{1{2 --}{2{2{2 the}{2{2{2 only}{3 good}}{2 thing}}}{2 --}}}}{2{2 about}{2{2 Extreme}{2 Ops}}}}{1{2{2 is}{2{2 that}{1{2 it}{0{2{2 ''s}{1{2 so}{1 inane}}}{2{2 that}{1{2 it}{1{2{2 gave}{3 me}}{1{2{2 plenty}{2{2 of}{2 time}}}{2{2 to}{2{2 ponder}{2{2 my}{2{2 Thanksgiving}{2{2 to-do}{1 list}}}}}}}}}}}}}}{2 .}}}'
    ),
    (
        10,
        '{0{2 You}{0{2 really}{1{0{2 have}{1{2 to}{1{3 wonder}{0{2 how}{2{2{2 on}{2{2 earth}{2 anyone}}}{1{2 ,}{0{2 anywhere}{1{2 could}{2{2 have}{1{2 thought}{2{2 they}{2{2 ''d}{2{2 make}{1{2{2 audiences}{1{2 guffaw}{2{2 with}{2{2 a}{2 script}}}}}{2{2 as}{3{2 utterly}{1{2 diabolical}{2{2 as}{2 this}}}}}}}}}}}}}}}}}}}{2 .}}}}'
    ),
    (
        11,
        '{1{1{2{2{2{2{2 Most}{2{2 of}{2{2 the}{2 information}}}}{1{2{2 has}{2 already}}{2{2 appeared}{3{2 in}{2{2{2{2 one}{2 forum}}{2 or}}{2 another}}}}}}{2 and}}{2 ,}}{1{2{1 no}{2{2 matter}{2{2 how}{3{2 Broomfield}{2{2{2 dresses}{2 it}}{2 up}}}}}}{1{2 ,}{2{2 it}{1{2 tends}{2{2 to}{1{1{2{2{2{2 speculation}{2 ,}}{2{2 conspiracy}{3 theories}}}{2 or}}{2{2 ,}{2{2{2 at}{4 best}}{2 ,}}}}{2{2 circumstantial}{2 evidence}}}}}}}}}{2 .}}'
    ),
    (
        12,
        '{1{3{2{3{2 The}{2 material}}{2 and}}{2{2 the}{2 production}}}{1{2 itself}{2{2{2 are}{1{2{2 little}{2 more}}{2{2 than}{2 routine}}}}{2 .}}}}'
    ),
    (
        9,
        '{1{2 Should}{2{2{2 have}{2{2 been}{2{2 someone}{2 else}}}}{2 -}}}'
    ),
    (
        11,
        '{2{3{3 Resident}{2 Evil}}{1{2{2 is}{1{2 what}{2{3 comes}{1{2 from}{2{2{2{2 taking}{2{2{2{2 John}{2{2 Carpenter}{2 ''s}}}{2 Ghosts}}{2{2 of}{2 Mars}}}}{2 and}}{2{2 eliminating}{2{2 the}{2 beheadings}}}}}}}}{2 .}}}'
    ),
    (
        12,
        '{3{4{4{2 A}{2{4 brilliant}{1{2 ,}{1{1 absurd}{2 collection}}}}}{2{2 of}{2{2 vignettes}{3{2 that}{2{2 ,}{3{2{2 in}{2{2 their}{3{2 own}{2{2 idiosyncratic}{2 way}}}}}{3{2 ,}{2{2{2{2 sum}{2 up}}{0{1{2 the}{2{1 strange}{1 horror}}}{2{2 of}{3 life}}}}{2{2 in}{2{2 the}{2{3 new}{2 millennium}}}}}}}}}}}}{2 .}}'
    ),
    (
        11,
        '{4{3{2 As}{3{2{2 giddy}{2 and}}{3{3 whimsical}{2{2 and}{3{2 relevant}{2 today}}}}}}{2{2{2 as}{2 it}}{2{2{2 was}{2{2{2 270}{2 years}}{2 ago}}}{2 .}}}}'
    ),
    (
        10,
        '{1{1{1{2{1 Coughs}{2 and}}{2 sputters}}{1{2 on}{2{2 its}{2{2 own}{1{2 postmodern}{1 conceit}}}}}}{2 .}}'
    ),
    (
        12,
        '{2{2{2 To}{2{2 my}{2 taste}}}{1{2 ,}{1{2{2{2 the}{2{2 film}{2 ''s}}}{2{2 comic}{2 characters}}}{1{1{2 come}{2{1 perilously}{2{2 close}{2{2 to}{2{2 being}{1{2{2{2 Amoses}{2 and}}{2 Andys}}{3{2 for}{3{2 a}{2{3 new}{2 generation}}}}}}}}}}{2 .}}}}}'
    ),
    (
        12,
        '{2{2 It}{1{1{2 ''s}{0{1{1 mindless}{2 junk}}{2{2 like}{2{2 this}{2{2 that}{2{2 makes}{3{2 you}{2{4{3 appreciate}{2{3 original}{2{2 romantic}{2 comedies}}}}{2{2 like}{2{2 Punch-Drunk}{4 Love}}}}}}}}}}}{2 .}}}'
    ),
    (
        9,
        '{4{2 Montias}{2{2 ...}{4{3{4{4{4{2 pumps}{3{2{2 a}{2 lot}}{2{2 of}{3 energy}}}}{3{2 into}{3{2 his}{3{3 nicely}{3{2 nuanced}{2 narrative}}}}}}{2 and}}{3{2{2 surrounds}{2 himself}}{3{2 with}{3{2{2 a}{2 cast}}{3{2 of}{3{2{2 quirky}{2{3{2{2 --}{2 but}}{3{1 not}{2 stereotyped}}}{2 --}}}{2{2 street}{2 characters}}}}}}}}{2 .}}}}'
    ),
    (
        11,
        '{2{2 Cattaneo}{2{2{2 should}{2{2 have}{2{3{2 followed}{4{4{2 the}{3{2 runaway}{3 success}}}{1{2 of}{2{2{2{2{2 his}{2{2 first}{2 film}}}{2 ,}}{2{2 The}{2{3 Full}{2 Monty}}}}{2 ,}}}}}{2{2 with}{3{2 something}{2 different}}}}}}{2 .}}}'
    ),
    (
        12,
        '{0{1{1{0{0{2 I}{1{3{2 ''m}{1 not}}{0{2 sure}{0{2 which}{1{2{2 half}{2{2 of}{3 Dragonfly}}}{0{1{1{2 is}{1 worse}}{2 :}}{1{2{1{2{2{2 The}{2 part}}{1{2 where}{0{1 nothing}{2{2 ''s}{2 happening}}}}}{2 ,}}{2 or}}{2{2{2 the}{2 part}}{3{2 where}{2{2 something}{2{2 ''s}{2 happening}}}}}}}}}}}}{2 ,}}{2 but}}{1{2 it}{1{2 ''s}{0 stupid}}}}{2 .}}'
    ),
    (
        9,
        '{3{2{2{2{2{2 The}{2 director}}{2 ,}}{2{2 Mark}{2 Pellington}}}{2 ,}}{2{1{2 does}{2{4{2 a}{3{4 terrific}{2 job}}}{0{2{2{2 conjuring}{2 up}}{2{2 a}{2{1 sinister}{2{2 ,}{1{1 menacing}{2 atmosphere}}}}}}{1{2 though}{0{1 unfortunately}{0{2{2 all}{2{2 the}{2 story}}}{2{2 gives}{2{3 us}{2{2 is}{2{2 flashing}{2{2{2{1{2{2{2 red}{2 lights}}{2 ,}}{2{2 a}{2{2 rattling}{2 noise}}}}{2 ,}}{2 and}}{1{2{2 a}{2 bump}}{2{2 on}{2{2 the}{2 head}}}}}}}}}}}}}}}{2 .}}}'
    ),
    (
        11,
        '{4{4{4{4{4 Astonishingly}{3{3{3 skillful}{2 and}}{3 moving}}}{2 ...}}{4{2 it}{4{2 could}{4{3{2 become}{3{2 a}{4{3{2 historically}{2 significant}}{2 work}}}}{3{2{2 as}{3 well}}{4{2 as}{4{3{2 a}{4 masterfully}}{2{2 made}{2 one}}}}}}}}}{2 .}}'
    ),
    (
        9,
        '{1{0{2 The}{0{1{0{2 most}{0 excruciating}}{2{2 86}{2 minutes}}}{2 one}}}{1{1{2 might}{1{2 sit}{1{2 through}{1{2{2 this}{2 summer}}{2{2 that}{2{2{2 do}{1 not}}{0{2 involve}{1{2 a}{2{2 dentist}{2 drill}}}}}}}}}}{2 .}}}'
    ),
    (
        9,
        '{4{2 ...}{4{4{3{3{4{2 a}{4{4 powerful}{2 sequel}}}{2 and}}{2 one}}{4{2 of}{4{3{2 the}{4{4 best}{2 films}}}{2{2 of}{2{2 the}{2 year}}}}}}{2 .}}}'
    ),
    (
        12,
        '{3{3{2{2 A}{2{2 deliberative}{2 account}}}{2{2 of}{2{2 a}{2 lifestyle}}}}{2{2{2 characterized}{2{2 by}{2{2{2{2 its}{2 surface-obsession}}{2 --}}{3{2 one}{2{2 that}{2{2 typifies}{2{2{2 the}{2 delirium}}{2{2 of}{2{2{2{2{2{2 post}{2 ,}}{2 pre}}{2 ,}}{2 and}}{3{2 extant}{2 stardom}}}}}}}}}}}{2 .}}}'
    ),
    (
        9,
        '{3{2{3{2 The}{2{2 fly-on-the-wall}{2 method}}}{2{2 used}{2{2 to}{2{3 document}{2{2 rural}{2{2 French}{2{2 school}{3 life}}}}}}}}{3{3{2 is}{3{3{2 a}{3{3 refreshing}{2 departure}}}{3{2 from}{2{3{2{2 the}{2{2{2{2 now}{2 more}}{2 prevalent}}{2 technique}}}{2{2 of}{2{2 the}{2 docu-makers}}}}{2{2 being}{2{2{2 a}{2{2 visible}{2 part}}}{2{2 of}{2{2 their}{2 work}}}}}}}}}{2 .}}}'
    ),
    (
        11,
        '{1{1{1{1{1{2{2 The}{2 plot}}{1{2 has}{1{2{2 a}{2 number}}{2{2 of}{2 holes}}}}}{2 ,}}{2 and}}{1{2{2 at}{2 times}}{1{2 it}{2{2{2 ''s}{2 simply}}{2 baffling}}}}}{2 .}}'
    ),
    (
        9,
        '{3{4{2{2{3{2 Bogdanovich}{3{2 puts}{3{2 history}{2{2 in}{2 perspective}}}}}{2 and}}{2 ,}}{3{4{2 via}{3{2{2 Kirsten}{2{2 Dunst}{2 ''s}}}{4{4 remarkable}{2 performance}}}}{4{2 ,}{3{2 he}{4{1 showcases}{3{2 Davies}{3{2 as}{4{2{2 a}{2{2 young}{2 woman}}}{3{2 of}{3{4 great}{3{3 charm}{3{2 ,}{3{2{3 generosity}{2 and}}{2 diplomacy}}}}}}}}}}}}}}{2 .}}'
    ),
    (
        12,
        '{2{2 This}{3{2{2{2 is}{2 mostly}}{2{2{2{3 well-constructed}{2 fluff}}{2 ,}}{2{2 which}{2{2 is}{2{2 all}{2{2 it}{2{2 seems}{2{2 intended}{2{2 to}{2 be}}}}}}}}}}{2 .}}}'
    ),
    (
        11,
        '{2{2{2{2{2{2 Ensemble}{2 movies}}{2 ,}}{2{2 like}{2{2 soap}{2 operas}}}}{2 ,}}{1{2{2 depend}{2{2 on}{3 empathy}}}{2 .}}}'
    ),
    (
        11,
        '{2{2{2{2{2 Would}{2 n''t}}{2 it}}{2{3{2 be}{3 funny}}{2{2 if}{2{2{2{2 a}{2 bunch}}{2{2 of}{2{2 Allied}{2 soldiers}}}}{2{2{2{2 went}{2 undercover}}{2{2 as}{2{2 women}{2{2 in}{2{2 a}{2{2 German}{2 factory}}}}}}}{3{2 during}{3{2 World}{2{1 War}{2 II}}}}}}}}}{2 ?}}'
    ),
    (
        9,
        '{3{2 It}{4{4{2 ''s}{4{3{2{2 a}{3{3{2 very}{3 tasteful}}{2 rock}}}{2 and}}{2{2 roll}{2 movie}}}}{2 .}}}'
    ),
    (
        9,
        '{0{2 ``}{0{2{2{2 The}{2 Adventures}}{2{2 of}{2{2 Pluto}{2 Nash}}}}{0{2 ''}{0{0{2 is}{1{2 a}{0{2 big}{0{2 time}{2 stinker}}}}}{2 .}}}}}'
    ),
    (
        11,
        '{2{2 There}{3{3{2 has}{2{2 to}{2{2 be}{3{3{2 a}{2{2 few}{2 advantages}}}{2{2 to}{3{2 never}{1{2 growing}{2 old}}}}}}}}{2 .}}}'
    ),
    (
        11,
        '{0{0{1{0{2{2 The}{2{2 following}{2 things}}}{0{2{2{2 are}{1 not}}{2{2 at}{2 all}}}{4 entertaining}}}{2 :}}{0{1{0{1{1{2 The}{1{0 bad}{2 sound}}}{2 ,}}{1{2{2 the}{2 lack}}{2{2 of}{1{2{2{3 climax}{2 and}}{2 ,}}{2{0 worst}{2{2 of}{2 all}}}}}}}{2 ,}}{3{2 watching}{2{2{2 Seinfeld}{2{1 -LRB-}{2{2{2 who}{2{2{2 is}{2 also}}{2{2 one}{2{2 of}{2{2{2 the}{2{2 film}{2 ''s}}}{2 producers}}}}}}{3 -RRB-}}}}{2{2 do}{2{2 everything}{2{2 he}{2{2 can}{2{2 to}{2{2 look}{3{2 like}{3{2 a}{3{3 good}{2 guy}}}}}}}}}}}}}}{2 .}}'
    ),
    (
        12,
        '{4{2{2 Preaches}{2{2 to}{2{2 two}{2{2 completely}{2 different}}}}}{4{3{2 choirs}{3{2 at}{3{2{2{2 the}{2{2 same}{2 time}}}{2 ,}}{3{2 which}{3{2 is}{4{2 a}{4{3{4 pretty}{4 amazing}}{3 accomplishment}}}}}}}}{2 .}}}'
    ),
    (
        10,
        '{1{2{2 The}{2 film}}{1{2{1{2 does}{2 n''t}}{2{2 show}{2{2{2 enough}{2{2 of}{2{2 the}{2{4 creative}{2{2{2 process}{2 or}}{2 even}}}}}}{3{2 of}{2{2 what}{3{2 was}{2{2{2 created}{2{2 for}{2{2 the}{2 non-fan}}}}{2{2 to}{2{2{2 figure}{1 out}}{2{2 what}{2{2 makes}{2{2 Wilco}{2{2 a}{2{2 big}{2 deal}}}}}}}}}}}}}}}{2 .}}}'
    ),
    (
        10,
        '{1{1{2{1{2{2 Payami}{2{2 tries}{2{2 to}{2{2 raise}{2{1{2 some}{2{2 serious}{2 issues}}}{2{2 about}{2{2{2 Iran}{2 ''s}}{2{2 electoral}{2 process}}}}}}}}}{2 ,}}{2 but}}{1{2{2 the}{2 result}}{1{2 is}{1{2{2 a}{2 film}}{1{2 that}{2{2{2 ''s}{2{2 about}{2{2 as}{3 subtle}}}}{2{2 as}{1{2 a}{2{2 party}{2{2 political}{3 broadcast}}}}}}}}}}}{2 .}}'
    ),
    (
        11,
        '{1{1{2{2{2{2 It}{3{2{2 ''s}{1 not}}{1{2 as}{1{2 awful}{1{2 as}{2{2 some}{1{2 of}{2{2 the}{1{2 recent}{2{2 Hollywood}{1{2 trip}{1 tripe}}}}}}}}}}}}{2 ...}}{2 but}}{1{2 it}{1{2 ''s}{0{2 far}{3{2 from}{4{2 a}{3{3 groundbreaking}{2 endeavor}}}}}}}}{2 .}}'
    ),
    (
        11,
        '{2{2{1{1{1{1{1{2 It}{1{2 ''s}{1{2 got}{1{2 some}{1{1 pretentious}{1{2 eye-rolling}{2 moments}}}}}}}{2 and}}{1{2 it}{1{2{2{2 did}{2 n''t}}{2 entirely}}{3{2 grab}{3 me}}}}}{2 ,}}{2 but}}{3{2 there}{3{2{3{2 ''s}{2 stuff}}{2 here}}{3{2 to}{2 like}}}}}{2 .}}'
    ),
    (
        10,
        '{1{1{2{2 Britney}{2{2 Spears}{2 ''}}}{2 phoniness}}{0{0{2 is}{0{1 nothing}{1{2 compared}{0{2 to}{0{0{0{2{2 the}{2{2 movie}{2 ''s}}}{1{2 contrived}{1{2 ,}{1{0 lame}{2 screenplay}}}}}{2 and}}{1{1 listless}{2 direction}}}}}}}{2 .}}}'
    ),
    (
        11,
        '{1{2 Clockstoppers}{1{1{2 is}{1{2 one}{1{2 of}{2{2{2 those}{1{2 crazy}{2{2 ,}{1{3 mixed-up}{2 films}}}}}{2{2 that}{2{1{2 does}{2 n''t}}{3{2 know}{2{2 what}{2{2 it}{2{3 wants}{2{2 to}{2{2 be}{2{2 when}{2{2 it}{3{3 grows}{2 up}}}}}}}}}}}}}}}}{2 .}}}'
    ),
    (
        10,
        '{4{3{2 While}{3{3{2{3{2{2 Parker}{2{2 and}{2{2 co-writer}{2{2 Catherine}{2{2 di}{2 Napoli}}}}}}{3{2{2{2 are}{3{3 faithful}{2{2 to}{2{2{2 Melville}{2 ''s}}{2 plotline}}}}}{2 ,}}{2 they}}}{2 and}}{3{2 a}{3{2{2 fully}{3 engaged}}{2{3 supporting}{2 cast}}}}}{3{2 ...}{3{2 have}{3{2 made}{3{2{2{2 the}{2{2 old}{2{2 boy}{2 ''s}}}}{2 characters}}{3{2 more}{3{3 quick-witted}{2{2 than}{2{2 any}{2{2 English}{2 Lit}}}}}}}}}}}}{2{2 major}{2{2{2 would}{3{2 have}{2{2 thought}{2 possible}}}}{2 .}}}}'
    ),
    (
        9,
        '{0{1{1{2 Loses}{2{2{2 its}{2 sense}}{2{2 of}{4 humor}}}}{0{2 in}{1{2{2 a}{2 vat}}{1{2 of}{0{0{1{1{1{1{0 failed}{2 jokes}}{2 ,}}{1{2 twitchy}{2 acting}}}{2 ,}}{2 and}}{1{2 general}{1 boorishness}}}}}}}{2 .}}'
    ),
    (
        12,
        '{1{2 It}{3{2{1{2 is}{1{2 too}{0 bad}}}{3{2 that}{2{3{2 this}{3{3 likable}{2 movie}}}{2{1{2 is}{2 n''t}}{3{2 more}{3 accomplished}}}}}}{2 .}}}'
    ),
    (
        10,
        '{3{2{2 Even}{2{2 in}{1{2 its}{0{1{2 most}{1 tedious}}{2 scenes}}}}}{3{2 ,}{4{2{2 Russian}{2 Ark}}{4{4{2 is}{4 mesmerizing}}{2 .}}}}}'
    ),
    (
        12,
        '{0{2 There}{2{2{2{2 are}{2 now}}{1{2{1{2{2 two}{2 signs}}{2{2 that}{1{2{2{2 M.}{2{2 Night}{2{2 Shyamalan}{2 ''s}}}}{2{2 debut}{2 feature}}}{2{1{0 sucked}{2{2 up}{2 all}}}{2{2 he}{2{2 has}{2{2 to}{2{3 give}{2{2 to}{2{2{2 the}{2{3 mystic}{2 genres}}}{2{2 of}{2 cinema}}}}}}}}}}}}{2 :}}{2{2{3 Unbreakable}{2 and}}{2 Signs}}}}{2 .}}}'
    ),
    (
        9,
        '{1{1{1{0{1{1{2 An}{2{1 inelegant}{2 combination}}}{0{2 of}{1{2{2 two}{1{1 unrelated}{2 shorts}}}{1{2 that}{0{1 falls}{1{2 far}{2 short}}}}}}}{2{2 of}{2{3{2 the}{2{2 director}{2 ''s}}}{2{2 previous}{2 work}}}}}{2{2 in}{2 terms}}}{4{2 of}{3{2 both}{3{2 thematic}{3{3 content}{3{2 and}{2{2 narrative}{3 strength}}}}}}}}{2 .}}'
    ),
    (
        10,
        '{1{2{2 Teen}{2 movies}}{1{1{2{2 have}{2 really}}{1{3 hit}{2{2 the}{2 skids}}}}{2 .}}}'
    ),
    (
        10,
        '{1{1{2{2 PC}{2 stability}}{2 notwithstanding}}{1{2 ,}{1{2{2 the}{2 film}}{2{0{2 suffers}{2{2 from}{2{3{2{2 a}{2{2 simplistic}{2 narrative}}}{2 and}}{2{2 a}{3{2 pat}{2{2 ,}{3{2 fairy-tale}{2 conclusion}}}}}}}}{2 .}}}}}'
    ),
    (
        9,
        '{3{2 But}{3{2 it}{3{3{2 could}{3{2{2{2{2 be}{2 ,}}{3{2 by}{3{2 its}{2{2{2 art}{2 and}}{2 heart}}}}}{2 ,}}{2{2 a}{2{2 necessary}{2 one}}}}}{2 .}}}}'
    ),
    (
        12,
        '{3{2{2 The}{2 film}}{3{3{3{3{2 feels}{3{2 uncomfortably}{3 real}}}{2 ,}}{2{2{2 its}{3{2{2 language}{2 and}}{2 locations}}}{3{2 bearing}{3{2{2 the}{2{2 unmistakable}{2 stamp}}}{2{2 of}{2 authority}}}}}}{2 .}}}'
    ),
    (
        12,
        '{4{2 It}{4{4{2 ''s}{2{3{2 a}{4{3 funny}{2{2 little}{2 movie}}}}{3{2 with}{4{4 clever}{3{2 dialogue}{3{2 and}{3{3 likeable}{2 characters}}}}}}}}{2 .}}}'
    ),
    (
        12,
        '{3{2{2 The}{2{2 first}{2{2 five}{2 minutes}}}}{4{4{2 will}{4{2 have}{2{2 you}{2{2 talking}{2{2 ''til}{2{2{2 the}{2 end}}{2{2 of}{2{2 the}{2 year}}}}}}}}}{2 !}}}'
    ),
    (
        11,
        '{1{2{2 If}{3{2 swimfan}{2{2 does}{2{2 catch}{2 on}}}}}{1{2 ,}{3{2 it}{2{2{2 may}{2{2 be}{2{2 because}{3{2 teens}{2{2 are}{2{2{2 looking}{2{2 for}{2 something}}}{2{2 to}{3{2 make}{2{2 them}{3 laugh}}}}}}}}}}{2 .}}}}}'
    ),
    (
        11,
        '{1{2{2{2{2{2 Director}{2{2 Elie}{2 Chouraqui}}}{2 ,}}{2{2 who}{2{2 co-wrote}{2{2 the}{2 script}}}}}{2 ,}}{1{1{3{2 catches}{1{2 the}{2{1 chaotic}{1 horror}}}}{2{1{2 of}{1 war}}{1{2{2 ,}{2 but}}{1{2 why}{0{2 bother}{1{2 if}{1{2 you}{1{2 ''re}{1{2 going}{2{2 to}{2{3{2 subjugate}{3 truth}}{2{2 to}{2{3{2 the}{2{3 tear-jerking}{2 demands}}}{2{2 of}{1{2 soap}{2 opera}}}}}}}}}}}}}}}}{2 ?}}}'
    ),
    (
        10,
        '{1{0{0{0{1 Excruciatingly}{1 unfunny}}{2 and}}{0{1 pitifully}{1 unromantic}}}{2 .}}'
    ),
    (
        10,
        '{3{3{2 A}{4{3 solid}{2 film}}}{2{2 ...}{2{2{2 but}{2{2{2{2 more}{2 conscientious}}{2 than}}{3{2 it}{3{3{2 is}{3 truly}}{3 stirring}}}}}{2 .}}}}'
    ),
    (
        9,
        '{2{3{2 This}{2{4 engrossing}{2{2 ,}{3{4{2 characteristically}{3 complex}}{2{2 Tom}{2{2 Clancy}{3 thriller}}}}}}}{1{1{2 is}{2{2 shifty}{3{2 in}{3{2{2 the}{2 manner}}{2{2{2 in}{2 which}}{2{2 it}{2{2{2{2 addresses}{2{2 current}{1{1 terrorism}{2 anxieties}}}}{2 and}}{2{2{2 sidesteps}{2 them}}{2{2 at}{2{2 the}{2{2 same}{2 time}}}}}}}}}}}}{2 .}}}'
    ),
    (
        11,
        '{3{1{2 Although}{1{2 it}{1{2 includes}{1{3{2 a}{2{2 fair}{3 share}}}{0{2 of}{0{1{1{1 dumb}{2{2 drug}{2 jokes}}}{2 and}}{2{2 predictable}{3 slapstick}}}}}}}}{3{2 ,}{4{2 ``}{3{2{2 Orange}{2 County}}{3{2 ''}{3{3{2{2 is}{3{2 far}{3 funnier}}}{2{2 than}{2{2 it}{2{2 would}{2{2 seem}{2{2 to}{2{2 have}{2{2 any}{3{3 right}{2{2 to}{2 be}}}}}}}}}}}{2 .}}}}}}}'
    ),
    (
        10,
        '{3{3{2 Seen}{2{2 in}{2{2 that}{2 light}}}}{2{2 ,}{3{2{2 Moonlight}{2 Mile}}{3{1{2 should}{2{2 strike}{2{2{2 a}{2 nerve}}{2{2 in}{2 many}}}}}{2 .}}}}}'
    ),
    (
        11,
        '{4{2{2 One}{2{2 of}{2{2 the}{2 year}}}}{2{3{2{2 ''s}{2 most}}{3{1 weirdly}{3{4 engaging}{2{2 and}{2{2 unpredictable}{2{2 character}{2 pieces}}}}}}}{2 .}}}'
    ),
    (
        9,
        '{0{1{0{1{1{1{2 It}{1{2{2 ''s}{2 also}}{1{1{0{1 not}{4 smart}}{2 or}}{2{2{2 barbed}{2 enough}}{2{2 for}{2{1 older}{2 viewers}}}}}}}{2 --}}{1 not}}{1{2 everyone}{1{2 thinks}{1{1{1 poo-poo}{2 jokes}}{2{2{2 are}{2 `}}{2 edgy}}}}}}{2 .}}{2 ''}}'
    ),
    (
        12,
        '{1{2 I}{1{1{2{2 did}{2 n''t}}{3{3{2 find}{2{2 much}{3 fascination}}}{2{2 in}{2{2 the}{2 swinging}}}}}{2 .}}}'
    ),
    (
        9,
        '{3{2{2 If}{1{2 you}{3{2{2 come}{2{2 from}{2{2{2 a}{3 family}}{2{2 that}{3{2{3{2{2{2{2{2{2{2{2 eats}{2 ,}}{2 meddles}}{2 ,}}{1 argues}}{2 ,}}{3 laughs}}{2 ,}}{2 kibbitzes}}{2 and}}{1 fights}}}}}}{2 together}}}}{3{2 ,}{3{2 then}{4{3{2 go}{4{2 see}{3{2 this}{4{4 delightful}{3 comedy}}}}}{2 .}}}}}'
    ),
    (
        12,
        '{1{1{1{0{2{2 It}{1{2 ''s}{0{1{1 leaden}{2 and}}{2 predictable}}}}{2 ,}}{2 and}}{1{3 laughs}{1{2 are}{1 lacking}}}}{2 .}}'
    ),
    (
        12,
        '{1{2{2{2{2 Less}{2{2 a}{2 study}}}{2{2 in}{2{2{2 madness}{2 or}}{4 love}}}}{2{2 than}{1{2{2 a}{2 study}}{2{2 in}{2{2 schoolgirl}{2 obsession}}}}}}{2 .}}'
    ),
    (
        10,
        '{1{2 But}{1{2{2{2 the}{2 feelings}}{2{2 evoked}{2{2 in}{2{2 the}{2 film}}}}}{1{2{2 are}{1{2{1{2 lukewarm}{2 and}}{2 quick}}{2{2 to}{2 pass}}}}{2 .}}}}'
    ),
    (
        9,
        '{2{2 It}{2{3{1{3 provides}{2{2 a}{1{2 grim}{2{2 ,}{1{1 upsetting}{2 glimpse}}}}}}{2{2 at}{2{2{2 the}{2 lives}}{2{2 of}{2{2 some}{2{2 of}{2{1{2 the}{2{2{2 1.2}{2 million}}{2 Palestinians}}}{2{2 who}{2{2 live}{2{2 in}{1{1{2 the}{2{2 crowded}{2{2 cities}{2{2 and}{2{2 refugee}{2 camps}}}}}}{2{2 of}{2 Gaza}}}}}}}}}}}}}{2 .}}}'
    ),
    (
        10,
        '{2{4{3 Hilarious}{2{2 musical}{3 comedy}}}{2{2 though}{1{1{2 stymied}{1{2 by}{1{2 accents}{1{2 thick}{2{2 as}{2 mud}}}}}}{2 .}}}}'
    ),
    (
        11,
        '{2{2 It}{2{4{3{2 ''s}{4{2 so}{3 good}}}{1{2 that}{3{2 you}{1{2{2 can}{2 practically}}{2{2 see}{1{2{2 the}{2{2 Hollywood}{2{2 `}{2{2 suits}{2 ''}}}}}{2{2{2{2 trying}{2{2 to}{2{2{2 put}{2 together}}{2{2 the}{2 cast}}}}}{2 and}}{2{2 filmmaking}{3{2{2{3 team}{2{2 for}{2{2 the}{2 all-too}}}}{2 -}}{2{2 inevitable}{2{2 American}{2 remake}}}}}}}}}}}}{2 .}}}'
    ),
    (
        9,
        '{2{2{2{2 Represents}{2 something}}{2{2 very}{2{2 close}{1{2 to}{1{2{2 the}{1 nadir}}{2{2 of}{2{2 the}{2 thriller\/horror}}}}}}}}{2{2 genre}{2 .}}}'
    ),
    (
        12,
        '{3{2 A}{3{3 stirring}{2{1 road}{2{2 movie}{2 .}}}}}'
    ),
    (
        12,
        '{4{3{2 An}{4 exquisitely}}{2{4{3{3 crafted}{2 and}}{2{2 acted}{2 tale}}}{2 .}}}'
    ),
    (
        11,
        '{4{2 Pacino}{2{4{3{2 is}{3{4 brilliant}{2{2 as}{2{1{2{2 the}{1{1 sleep-deprived}{2 Dormer}}}{2 ,}}{2{2{2 his}{2{2 increasing}{1 weariness}}}{2{2 as}{3{2 much}{2 existential}}}}}}}}{2{2 as}{2{2 it}{2{2 is}{2 physical}}}}}{2 .}}}'
    ),
    (
        12,
        '{2{2{4 Clever}{1{2{2 but}{1 not}}{3{2 especially}{3 compelling}}}}{2 .}}'
    ),
    (
        10,
        '{1{2 ...}{2{2{2 while}{3{2{2{2 each}{3 moment}}{2{2 of}{1{2 this}{1{2 broken}{2{2 character}{2 study}}}}}}{3{2 is}{4{3 rich}{2{2 in}{2{2 emotional}{2 texture}}}}}}}{1{2 ,}{0{2{2 the}{2 journey}}{1{1{2{1{2 does}{2 n''t}}{2 really}}{2{2 go}{2 anywhere}}}{2 .}}}}}}'
    ),
    (
        10,
        '{1{1{2{2 Shaky}{2 close-ups}}{2{2 of}{2{1{2{1{1{2{2{1{2 turkey-on-rolls}{2 ,}}{2{2 stubbly}{2 chins}}}{2 ,}}{2{2 liver}{2 spots}}}{2 ,}}{2{2 red}{2 noses}}}{2 and}}{2{2 the}{2{2 filmmakers}{2{3 new}{2 bobbed}}}}}}}{2{1{2 do}{1{2{2{2 draw}{2{2 easy}{1 chuckles}}}{2 but}}{1{2 lead}{2 nowhere}}}}{2 .}}}'
    ),
    (
        12,
        '{1{1{2 Animated}{1 drivel}}{0{1{2 meant}{1{2 to}{2{3 enhance}{1{3{2 the}{2 self-image}}{1{2 of}{0{2 drooling}{0 idiots}}}}}}}{2 .}}}'
    ),
    (
        9,
        '{1{2{1 Forget}{1{2 the}{2{1 misleading}{2 title}}}}{1{2 ,}{2{2 what}{2{1{2 ''s}{1{2 with}{2{2 the}{2{2 unexplained}{2{2 baboon}{2 cameo}}}}}}{2 ?}}}}}'
    ),
    (
        10,
        '{1{3{2 Works}{2{2 hard}{2{2 to}{2 establish}}}}{1{2{2{2{2{2{2 rounded}{2 characters}}{2 ,}}{2 but}}{2 then}}{1{0{2 has}{1{1 nothing}{2{2{4 fresh}{2 or}}{3{2 particularly}{3 interesting}}}}}{2{2 to}{2{2 say}{2{2 about}{2 them}}}}}}{2 .}}}'
    ),
    (
        12,
        '{3{2 A}{3{3{3{3 treat}{2{2 for}{2{2 its}{2 depiction}}}}{2{2 on}{2{1 not}{2{1{2 giving}{2 up}}{2{2 on}{2{2 dreams}{1{2 when}{1{2 you}{2{2 ''re}{1{2 a}{1{1 struggling}{2 nobody}}}}}}}}}}}}{2 .}}}'
    ),
    (
        12,
        '{4{2{2 Those}{2{2 who}{2{1{2 are}{2 n''t}}{1{2{2 put}{1 off}}{2{2 by}{2{2{2 the}{2{2 film}{2 ''s}}}{2 austerity}}}}}}}{3{3{2 will}{3{2{2 find}{2 it}}{3{3{2 more}{2{2 than}{3 capable}}}{2{2 of}{2{3 rewarding}{2 them}}}}}}{2 .}}}'
    ),
    (
        11,
        '{1{1{0{0{2 A}{1{1 beyond-lame}{2 satire}}}{2 ,}}{0{3{3{2 Teddy}{2{2 Bears}{2 ''}}}{2{2 Picnic}{2 ranks}}}{1{2 among}{0{2 the}{1{1{2 most}{1 pitiful}}{2 directing}}}}}}{3{3{2 debuts}{3{2 by}{3{2 an}{3{4 esteemed}{2 writer-actor}}}}}{2 .}}}'
    ),
    (
        9,
        '{2{2{2 When}{2{2{2{2 the}{2{3 precise}{2 nature}}}{2{2 of}{3{2{2 Matthew}{2 ''s}}{1 predicament}}}}{3{2 finally}{3{3 comes}{2{2 into}{3{3 sharp}{2 focus}}}}}}}{2{2 ,}{1{2{2 the}{3 revelation}}{1{1{0 fails}{2{2 to}{3{2 justify}{2{2 the}{2 build-up}}}}}{2 .}}}}}'
    ),
    (
        10,
        '{3{3{2 By}{2{2 turns}{4{3 gripping}{4{2 ,}{4{4 amusing}{3{2 ,}{3{3{2 tender}{2 and}}{2 heart-wrenching}}}}}}}}{4{2 ,}{3{2 Laissez-passer}{3{4{2 has}{3{2{2 all}{3{2 the}{2 earmarks}}}{4{2 of}{4{2{2 French}{2 cinema}}{4{2 at}{3{2 its}{4 best}}}}}}}{2 .}}}}}'
    ),
    (
        11,
        '{0{2{2 A}{2 movie}}{0{0{2 more}{1{2 to}{0{2 be}{1{3 prescribed}{3{2 than}{1{1{0{0{1{3{4 recommended}{2 --}}{1{1{2 as}{1{3 visually}{1 bland}}}{1{2 as}{1{2{2 a}{2{2 dentist}{2 ''s}}}{2{2 waiting}{2 room}}}}}}{2 ,}}{2{3 complete}{2{2 with}{2{2 soothing}{2 Muzak}}}}}{2 and}}{2{2{2 a}{2 cushion}}{1{2 of}{2{2 predictable}{2{2 narrative}{2 rhythms}}}}}}}}}}}{2 .}}}'
    ),
    (
        9,
        '{3{2 There}{3{3{2 is}{3{3{3{2{3{2{2 a}{2 fabric}}{2{2 of}{2{3 complex}{2 ideas}}}}{2 here}}{2 ,}}{2 and}}{3{2 feelings}{2{2 that}{4{3 profoundly}{2{2 deepen}{2 them}}}}}}}{2 .}}}'
    ),
    (
        10,
        '{0{1{3{3{2{2{2 The}{2 experience}}{2{2 of}{2{2 going}{2{2 to}{2{2 a}{3{2 film}{3 festival}}}}}}}{3{2 is}{3{2 a}{3{3 rewarding}{2 one}}}}}{2 ,}}{2{2{2{2 the}{2 experiencing}}{2{2 of}{2{2{2 sampling}{2 one}}{2{2 through}{2{2 this}{2 movie}}}}}}{1{2 is}{1 not}}}}{2 .}}'
    ),
    (
        10,
        '{2{1{3 Does}{1{2 little}{2{2 to}{1{2 elaborate}{1{2{2 the}{1 conceit}}{2{2 of}{2{2{2 setting}{2{2{2 this}{2{2 blood-soaked}{1 tragedy}}}{2{2 of}{3{1 murderous}{3 ambition}}}}}{2{2 in}{3{2{2 the}{2 era}}{3{2 of}{2{2 Richard}{2 Nixon}}}}}}}}}}}}{2 .}}'
    ),
    (
        11,
        '{2{2{2{2 Every}{2{2 child}{2 ''s}}}{2 story}}{3{2{2 is}{2{2 what}{2 matters}}}{2 .}}}'
    ),
    (
        11,
        '{3{1 Formuliac}{3{2 ,}{3{3{2 but}{4 fun}}{2 .}}}}'
    ),
    (
        10,
        '{2{2 It}{0{1{2{1{2{2 tells}{2{2 its}{2 story}}}{1{2 in}{2{2 a}{1{1 flat}{2 manner}}}}}{2 and}}{1{2{2 leaves}{2 you}}{1{2 with}{2{2{2 the}{3 impression}}{1{2 that}{2{2 you}{1{2 should}{2{2 have}{2{2{2{2 gotten}{2{2 more}{1 out}}}{2{2 of}{2 it}}}{2{2 than}{2{2 you}{2 did}}}}}}}}}}}}{2 .}}}'
    ),
    (
        12,
        '{3{2 But}{3{2 it}{3{2 also}{3{3{2 has}{3{2 many}{2{2 of}{3{2{2 the}{2 things}}{2{2 that}{3{2 made}{3{2{2 the}{2{2 first}{2 one}}}{4 charming}}}}}}}}{2 .}}}}}'
    ),
    (
        12,
        '{3{3 Harks}{3{2{2{2 back}{2{2 to}{2{2 a}{2 time}}}}{2{2 when}{2{2 movies}{3{2{2 had}{2 more}}{1{2 to}{2{3{2 do}{3{2 with}{3 imagination}}}{2{2 than}{2{2 market}{2 research}}}}}}}}}{2 .}}}'
    ),
    (
        9,
        '{0{2 Manages}{1{0{2 to}{1{2 be}{0{2 both}{1{1{0{1 repulsively}{1 sadistic}}{2 and}}{1 mundane}}}}}{2 .}}}'
    ),
    (
        10,
        '{1{0{1{1{1{2 Seems}{1{3 content}{1{2 to}{1{2 dog-paddle}{2{2 in}{1{0{2 the}{1{1 mediocre}{2 end}}}{2{2 of}{2{2 the}{2 pool}}}}}}}}}{2 ,}}{2 and}}{1{2 it}{1{2 ''s}{2{2 a}{1{1 sad}{2{2 ,}{1{1 sick}{2 sight}}}}}}}}{2 .}}'
    ),
    (
        12,
        '{3{3{3{3{3{2 This}{3{2 is}{2{2{2{2 the}{3 kind}}{2{2 of}{2{2 subject}{2 matter}}}}{3{2 that}{3{2{2 could}{2{2 so}{3 easily}}}{2{2 have}{1{2 been}{2{1 fumbled}{2{2 by}{2{2 a}{2{2 lesser}{2 filmmaker}}}}}}}}}}}}{2 ,}}{2 but}}{4{2 Ayres}{3{2 makes}{3{3{2 the}{3{3 right}{2 choices}}}{2{2 at}{2{2 every}{2 turn}}}}}}}{2 .}}'
    ),
    (
        9,
        '{0{0{0{0{1{1{2 This}{0{2{1{2 is}{2{2 rote}{2 spookiness}}}{2 ,}}{0{2 with}{0{2{0{2 nary}{3{2 an}{2{3 original}{2 idea}}}}{1{1{2{1 -LRB-}{2 or}}{3{2{2{2{2 role}{2 ,}}{2{2{2{2{2{2 or}{1 edit}}{2 ,}}{2 or}}{3 score}}{2 ,}}}{2 or}}{2{2{2 anything}{2 ,}}{2 really}}}}{3 -RRB-}}}{2{2 in}{2 sight}}}}}}{2 ,}}{2 and}}{2{2{2{2 the}{2 whole}}{2{2 of}{2{2 the}{2 proceedings}}}}{1{2{2{2 beg}{2{2 the}{2 question}}}{2 `}}{2 Why}}}}{2 ?}}{2 ''}}'
    ),
    (
        12,
        '{4{4{3{4{2 Move}{2{2 over}{2 Bond}}}{2 ,}}{3{3{2 this}{2 girl}}{4{2 deserves}{2{2 a}{2 sequel}}}}}{2 .}}'
    ),
    (
        12,
        '{0{2 Personally}{0{2 ,}{1{2 I}{1{1{2{2 ''d}{2 rather}}{2{2{2 watch}{2 them}}{2{2 on}{2{2 the}{2{2 Animal}{2 Planet}}}}}}{2 .}}}}}'
    ),
    (
        12,
        '{3{4{3{2 A}{4{3 thought-provoking}{3{2 and}{3{3 often-funny}{1 drama}}}}}{2{2 about}{1 isolation}}}{2 .}}'
    ),
    (
        11,
        '{1{2{2 The}{2 story}}{1{2 itself}{2{2{3{2 is}{2 actually}}{1{2 quite}{1 vapid}}}{2 .}}}}'
    ),
    (
        10,
        '{1{2{2 Some}{3 fine}}{1{1{2{2 acting}{2{2{2 ,}{2 but}}{2 ultimately}}}{0{2{2 a}{2 movie}}{1{2 with}{1{2{1 no}{2 reason}}{2{2 for}{2 being}}}}}}{2 .}}}'
    ),
    (
        10,
        '{2{2{1{2{1{0{0{2{2 None}{2{2 of}{2{2 his}{2 actors}}}}{3{2 stand}{1 out}}}{2 ,}}{2 but}}{2{2 that}{2{2{2{2 ''s}{2{2 less}{2{2 of}{2{2 a}{1 problem}}}}}{2 here}}{2{2 than}{2{2 it}{2{2 would}{2{2 be}{2{2 in}{2{2 another}{2 film}}}}}}}}}}{2 :}}{2{2 Characterization}{2{2 matters}{2{2 less}{2{2 than}{2 atmosphere}}}}}}{2 .}}'
    ),
    (
        10,
        '{1{2 ...}{0{0{2 expands}{1{2{2 the}{3 horizons}}{0{2 of}{0{0{0{1{0{0{0 boredom}{2{2 to}{2{2 the}{2 point}}}}{2{2 of}{1 collapse}}}{2 ,}}{1{2 turning}{0{2 into}{0{1{2 a}{2{2 black}{2 hole}}}{2{2 of}{1 dullness}}}}}}{2 ,}}{1{2{2 from}{2 which}}{1{0{1 no}{3{3 interesting}{2 concept}}}{2{2 can}{3 escape}}}}}}}}{2 .}}}'
    ),
    (
        10,
        '{3{4{3{4{2{2 Not}{2{2 a}{2 film}}}{3{2 for}{2{2 the}{3{2{1{2 faint}{3{2 of}{2{3{2{2 heart}{2 or}}{2 conservative}}{3{2 of}{3 spirit}}}}}{2{2{2 ,}{2 but}}{2{2 for}{2{2{2 the}{2 rest}}{2{2 of}{2{2{2{3 us}{2 --}}{2 especially}}{2{2 San}{2 Francisco}}}}}}}}{2 lovers}}}}}{2 --}}{4{2 it}{3{2 ''s}{4{3{3{2 a}{3{4 spirited}{2 film}}}{2 and}}{4{2 a}{4 must-see}}}}}}{2 .}}'
    ),
    (
        10,
        '{2{2{2{3{2 Matches}{1{2 neorealism}{2 ''s}}}{2 impact}}{2{2 by}{3{2 showing}{2{2{2 the}{2 humanity}}{2{2 of}{2{2 a}{2{2 war-torn}{2 land}}}}}}}}{2{3{2 filled}{2{2 with}{2{2 people}{2{2 who}{2{2 just}{2{2 want}{2{2 to}{2{2 live}{2{2 their}{2 lives}}}}}}}}}}{2 .}}}'
    ),
    (
        11,
        '{1{1{1{1{1{2{3{2 Costner}{2 ''s}}{2{2 warm-milk}{2 persona}}}{0{2{2 is}{2 just}}{1{2 as}{2{1 ill-fitting}{2{2 as}{1{2{2 Shadyac}{2 ''s}}{2{2 perfunctory}{2{2 directing}{2 chops}}}}}}}}}{2 ,}}{2 and}}{1{1{2 some}{1{2 of}{1{2 the}{1{2 more}{1{1{2 overtly}{3 silly}}{2 dialogue}}}}}}{1{2 would}{2{2 sink}{2{2 Laurence}{2 Olivier}}}}}}{2 .}}'
    ),
    (
        10,
        '{4{4{2 Provides}{3{3{2 a}{3{3{2 very}{4{2{3 moving}{2 and}}{3 revelatory}}}{2 footnote}}}{2{2 to}{1{2 the}{2 Holocaust}}}}}{2 .}}'
    ),
    (
        12,
        '{3{3{3{2{2 It}{1{2 ''s}{3{2 about}{2{2 issues}{2{2{2 most}{2 adults}}{2{2 have}{2{2 to}{2{2 face}{3{2 in}{2 marriage}}}}}}}}}}{2 and}}{2{2 I}{3{2 think}{3{3{2 that}{2{2 ''s}{3{2 what}{3{2 I}{2{3 liked}{2{2 about}{2 it}}}}}}}{1{2 --}{1{3{2 the}{2{3 real}{2 issues}}}{2{2 tucked}{2{2 between}{1{2 the}{1{3 silly}{0{2 and}{1{1 crude}{2 storyline}}}}}}}}}}}}}{2 .}}'
    ),
    (
        11,
        '{4{3{2 That}{3{2{2 the}{2{3 real}{2{2 Antwone}{2 Fisher}}}}{3{2 was}{4{3 able}{3{2 to}{3{3{2{2 overcome}{3{2 his}{2{3 personal}{1 obstacles}}}}{2 and}}{2{2 become}{2{2 a}{3{3 good}{2 man}}}}}}}}}}{3{3{3{3{2 is}{4{2 a}{4{3 wonderful}{2 thing}}}}{2 ,}}{4{2 that}{4{2 he}{2{2 has}{3{2 been}{4{3 able}{3{2 to}{3{3 share}{3{2{2 his}{2 story}}{3{3{2 so}{3 compellingly}}{3{2 with}{3{3 us}{2{2 is}{3{2 a}{3{2 minor}{3 miracle}}}}}}}}}}}}}}}}{2 .}}}'
    ),
    (
        9,
        '{4{2{2 The}{2 film}}{3{3{2 boasts}{2{3{2{3{2{3{1 dry}{4 humor}}{2 and}}{3{2 jarring}{2 shocks}}}{2 ,}}{3 plus}}{3{2 moments}{4{2 of}{3{4 breathtaking}{2 mystery}}}}}}{2 .}}}'
    ),
    (
        11,
        '{3{2{1 Never}{2 Again}}{3{2 ,}{3{2{2 while}{1{1 nothing}{3 special}}}{3{2 ,}{4{3{3{2 is}{3{4 pleasant}{2{2 ,}{3{2{2 diverting}{2 and}}{3 modest}}}}}{4{2 --}{3{3 definitely}{3{2{2 a}{2 step}}{3{2 in}{3{2 the}{3{3 right}{2 direction}}}}}}}}{2 .}}}}}}'
    ),
    (
        10,
        '{4{2{2 Woody}{2 Allen}}{3{4{2{2 has}{2 really}}{3{3{2 found}{2{2 his}{2 groove}}}{2{2 these}{2 days}}}}{2 .}}}'
    ),
    (
        9,
        '{1{1{2 Contains}{1{2{2 all}{2{2 the}{2 substance}}}{2{2 of}{2{2{2 a}{2 Twinkie}}{2{2 --}{1{2{3{2{2 easy}{2{2 to}{2 swallow}}}{2 ,}}{2 but}}{2{2 scarcely}{3 nourishing}}}}}}}}{2 .}}'
    ),
    (
        11,
        '{0{2 Just}{0{1{2 plain}{0 bad}}{2 .}}}'
    ),
    (
        9,
        '{0{0{1{2{2{2{2 A}{1 Rumor}}{2{2 of}{2 Angels}}}{2{2{1{2 does}{2 n''t}}{2 just}}{2 slip}}}{2 --}}{2{2 it}{2{2 avalanches}{3{2 into}{2{1 forced}{1 fuzziness}}}}}}{2 .}}'
    ),
    (
        9,
        '{4{4{2{3{2 It}{2{2 ''s}{3{1 no}{1 lie}}}}{2 --}}{4{2{2 Big}{2{1 Fat}{1 Liar}}}{3{2 is}{4{2 a}{2{3 real}{3 charmer}}}}}}{2 .}}'
    ),
    (
        9,
        '{4{4{3{4{2 Congrats}{3 Disney}}{2{2 on}{2{2 a}{2 job}}}}{4{3 well}{2 done}}}{3{2 ,}{4{2 I}{4{4{3{3{3 enjoyed}{2 it}}{2 just}}{2{2 as}{2 much}}}{2 !}}}}}'
    ),
    (
        10,
        '{4{4{2{3{3{2{2{2 Martin}{2 and}}{2 Barbara}}{3{2 are}{2{3{3{3 complex}{2 characters}}{3{2 --}{3{2 sometimes}{3{2 tender}{2 ,}}}}}{1{2 sometimes}{1 angry}}}}}{2 --}}{2 and}}{4{3{3{2 the}{3{3 delicate}{2 performances}}}{2{2 by}{2{2{2{2 Sven}{2 Wollter}}{2 and}}{2{2 Viveka}{2 Seldahl}}}}}{2{2 make}{3{2{2 their}{2{2{2 hopes}{2 and}}{1 frustrations}}}{3 vivid}}}}}{2 .}}'
    ),
    (
        10,
        '{2{3 Very}{3{3 well}{2{1{2{2{2 made}{2 ,}}{2 but}}{2{1{2 does}{2 n''t}}{2{2 generate}{3{2{2 a}{2 lot}}{2{2 of}{2 tension}}}}}}{2 .}}}}'
    ),
    (
        11,
        '{0{1{2 What}{0{0{2 a}{0{1 stiflingly}{1{1 unfunny}{1{2 and}{1{1 unoriginal}{1 mess}}}}}}{2 this}}}{2{2 is}{2 !}}}'
    ),
    (
        9,
        '{3{3{2{3{3{3{2{2 The}{2 huskies}}{4{2 are}{4 beautiful}}}{2 ,}}{3{2{2 the}{2{2 border}{2 collie}}}{3{2 is}{3 funny}}}}{2 and}}{3{2{2 the}{2{2 overall}{2 feeling}}}{4{2 is}{3{3{3 genial}{2 and}}{3 decent}}}}}{2 .}}'
    ),
    (
        12,
        '{1{3{3{2 For}{3{2 all}{3{2 its}{3{4 impressive}{2 craftsmanship}}}}}{2{3{2 ,}{2 and}}{2{2 despite}{2{1{2 an}{1{1 overbearing}{2 series}}}{2{2 of}{2{2 third-act}{2 crescendos}}}}}}}{1{2 ,}{1{2{2 Lily}{2 Chou-Chou}}{1{2 never}{3{3{2 really}{3{2{2 builds}{2 up}}{2{2{2 a}{2 head}}{2{2 of}{2{2 emotional}{2 steam}}}}}}{2 .}}}}}}'
    ),
    (
        9,
        '{1{2{2 In}{2{2 other}{2 words}}}{1{2 ,}{2{2 it}{2{2{2{2 ''s}{2 just}}{2{2 another}{2{2 sports}{2{2 drama\/character}{2 study}}}}}{2 .}}}}}'
    ),
    (
        10,
        '{4{2{3 Like}{2{2 The}{2{3 Full}{2 Monty}}}}{4{2 ,}{4{2 this}{3{3{2 is}{4{2 sure}{4{2 to}{4{3{3{2 raise}{2{2{2 audience}{2 ''s}}{2 spirits}}}{2 and}}{3{1 leave}{2{2 them}{3{2{2 singing}{2 long}}{2{2 after}{2{2 the}{2{2 credits}{2 roll}}}}}}}}}}}{2 .}}}}}'
    ),
    (
        12,
        '{1{3{2{2 A}{3{3 well-intentioned}{2 effort}}}{1{2 that}{1{2{2 ''s}{2 still}}{1{2 too}{1{2 burdened}{2{2 by}{2{2 the}{2 actor}}}}}}}}{2{2{2 ''s}{1{2{3{2 offbeat}{2 sensibilities}}{3{2 for}{3{2 the}{3{2 earnest}{2{2 emotional}{2 core}}}}}}{2{2 to}{2{2 emerge}{2{2 with}{2{2{2 any}{2 degree}}{2{2 of}{2 accessibility}}}}}}}}{2 .}}}'
    ),
    (
        10,
        '{2{1{2 If}{1{2 you}{3{2 can}{1{2{2 tolerate}{2{2 the}{2{2 redneck-versus-blueblood}{2 cliches}}}}{2{2 that}{2{2{2 the}{2 film}}{2{2 trades}{2 in}}}}}}}}{2{2 ,}{2{2{3 Sweet}{4{3 Home}{2 Alabama}}}{2{2{2 is}{2{2 diverting}{2{2 in}{2{2{2 the}{2 manner}}{2{2 of}{2{2{2 Jeff}{2{2 Foxworthy}{2 ''s}}}{2{2 stand-up}{2 act}}}}}}}}{2 .}}}}}'
    ),
    (
        10,
        '{1{2 Big}{1{2 mistake}{2 .}}}'
    ),
    (
        11,
        '{1{1{2{3{3{2 A}{3 hit}}{2 -}}{2{2 and-miss}{1 affair}}}{2 ,}}{2{2{3{3{3{2 consistently}{4 amusing}}{2 but}}{1{1 not}{2{2 as}{3{2{2 outrageous}{2 or}}{3 funny}}}}}{2{2{2{2 as}{2{2 Cho}{2{2 may}{2{2 have}{2 intended}}}}}{2 or}}{3{2 as}{3{2{4 imaginative}{2{2 as}{2 one}}}{2{2 might}{2{2 have}{2 hoped}}}}}}}{2 .}}}'
    ),
    (
        12,
        '{2{2 It}{2{2{2 ''s}{2{2{2 another}{1 retelling}}{2{2 of}{2{2{3 Alexandre}{2{2 Dumas}{2 ''}}}{3 classic}}}}}{2 .}}}'
    ),
    (
        9,
        '{3{4{2 Psychologically}{3 savvy}}{2 .}}'
    ),
    (
        12,
        '{4{3{4{4{4{4{4 Great}{2 performances}}{2 ,}}{3{3 stylish}{2 cinematography}}}{2 and}}{2{2 a}{2{3 gritty}{2{2 feel}{2 help}}}}}{3{4{2 make}{4{2{2 Gangster}{4{1 No.}{2 1}}}{3{2 a}{3{3 worthwhile}{2{2 moviegoing}{2 experience}}}}}}{2 .}}}'
    ),
    (
        11,
        '{0{2{2 A}{2 peculiar}}{1{0{1 misfire}{1{2{2 that}{2 even}}{2{2 Tunney}{2{2{2 ca}{2 n''t}}{2 save}}}}}{2 .}}}'
    ),
    (
        11,
        '{1{2 Yet}{1{1{2 why}{0{2 it}{0 fails}}}{2{2{2 is}{3{2{2 a}{2 riddle}}{2{2 wrapped}{2{2 in}{2{2{2 a}{2 mystery}}{3{2 inside}{2{2 an}{2 enigma}}}}}}}}{2 .}}}}'
    ),
    (
        11,
        '{4{2 ``}{4{2{2{2 13}{2 Conversations}}{2{2 About}{2{2 One}{2 Thing}}}}{4{2 ''}{4{4{2 is}{4{3{2 an}{4{3 intelligent}{2 flick}}}{3{2 that}{3{3{2 examines}{3{2{2 many}{2{2 different}{2 ideas}}}{2{2 from}{4 happiness}}}}{2{2 to}{3{1 guilt}{3{2 in}{3{3{2 an}{3{3 intriguing}{2 bit}}}{2{2 of}{2 storytelling}}}}}}}}}}{2 .}}}}}'
    ),
    (
        11,
        '{3{2 ``}{3{2 Frailty}{4{2 ''}{3{4{3{2 has}{3{2 been}{3{4{3{3 written}{3{2 so}{3 well}}}{2 ,}}{2{2{2 that}{2 even}}{1{2{2 a}{2 simple}}{1{2 ``}{0 Goddammit}}}}}}}{2 !}}{2 ''}}}}}'
    ),
    (
        11,
        '{1{1{2 Despite}{2{2{2 Juliet}{2{2 Stevenon}{2 ''s}}}{1{2 attempt}{2{2 to}{2{3{3 bring}{2 cohesion}}{3{2 to}{2{2{2 Pamela}{2 ''s}}{2{2 emotional}{2{2 roller}{2{2 coaster}{3 life}}}}}}}}}}}{1{2 ,}{1{2 it}{1{1{1{2 is}{1 not}}{1{2 enough}{1{2 to}{2{2{3 give}{2{2 the}{2 film}}}{1{2{2 the}{2 substance}}{2{2 it}{2{2{2 so}{1 desperately}}{2 needs}}}}}}}}{2 .}}}}}'
    ),
    (
        9,
        '{2{3{2 Count}{3{2 on}{2{2 his}{2{2 movie}{2{2 to}{2{2{2 work}{3{2 at}{2{2{2 the}{2 back}}{2{2 of}{2{2 your}{2 neck}}}}}}{3{2 long}{2{2 after}{2{2 you}{1{1 leave}{2{2 the}{3 theater}}}}}}}}}}}}{2 .}}'
    ),
    (
        11,
        '{3{3{2{2 Not}{2{2 about}{2 scares}}}{3{2 but}{3{2{2 a}{2 mood}}{3{2{2 in}{2 which}}{2{2{2 an}{2{2 ominous}{2{2 ,}{2{2{3 pervasive}{2{3{2 ,}{2 and}}{2 unknown}}}{2 threat}}}}}{3{1{2{2{2 lurks}{2 just}}{2{1 below}{2{2 the}{2 proceedings}}}}{2 and}}{3{2 adds}{3{2{2 an}{2{2{2 almost}{2 constant}}{2 mindset}}}{2{2 of}{2 suspense}}}}}}}}}}{2 .}}'
    ),
    (
        9,
        '{3{3{2{2 A}{2 film}}{3{2 of}{3{2 quiet}{2 power}}}}{2 .}}'
    ),
    (
        9,
        '{3{2 VeretA(C)}{3{3{2 has}{3{2{2 a}{2{3 whip-smart}{2 sense}}}{2{2 of}{2{2 narrative}{2 bluffs}}}}}{2 .}}}'
    ),
    (
        11,
        '{2{2{2{2 Most}{2 folks}}{2{2 with}{2{2{2 a}{2{3 real}{2 stake}}}{2{2 in}{3{2 the}{2{2 American}{2{2 sexual}{2 landscape}}}}}}}}{2{1{2 will}{1{2 find}{1{2 it}{1{3{3{2 either}{3{2 moderately}{4 amusing}}}{2 or}}{0{2 just}{1{2 plain}{1 irrelevant}}}}}}}{2 .}}}'
    ),
    (
        10,
        '{1{2{2{2 The}{2{2 movie}{2 ''s}}}{2{2 something-borrowed}{2 construction}}}{1{1{2 feels}{1{2 less}{2{2{3{2{2{3{2 the}{2 product}}{3{2 of}{2 loving}}}{2 ,}}{2{3{3 well}{2 integrated}}{3 homage}}}{2 and}}{0{2 more}{1{2 like}{0{1{2 a}{2{2 mere}{2 excuse}}}{1{2 for}{1{2{2{2 the}{1 wan}}{2 ,}}{1{2{2 thinly}{2 sketched}}{2 story}}}}}}}}}}{2 .}}}'
    ),
    (
        12,
        '{2{2{2{2{1{2 Ritchie}{1{1{2 may}{1 not}}{2{2 have}{2{2{2 a}{3 novel}}{2{2 thought}{2{2 in}{2{2 his}{2 head}}}}}}}}{2 ,}}{2 but}}{2{2 he}{2{2 knows}{2{2 how}{2{2 to}{2{2 pose}{2 Madonna}}}}}}}{2 .}}'
    ),
    (
        12,
        '{2{2 It}{1{1{2 ''s}{1{2 too}{1{0 bad}{2{2{1 nothing}{2 else}}{2 is}}}}}{2 .}}}'
    ),
    (
        9,
        '{4{4{4{2 The}{4{3 gorgeously}{3{2 elaborate}{2 continuation}}}}{2{2{2 of}{2 ``}}{2{2 The}{2{2{2 Lord}{2{2 of}{2{2 the}{2 Rings}}}}{2{2 ''}{2 trilogy}}}}}}{2{3{2{2 is}{2{2 so}{2 huge}}}{2{2 that}{3{2{2{2 a}{2 column}}{2{2 of}{2 words}}}{2{2{2{2 can}{1 not}}{3 adequately}}{2{2 describe}{2{3{2{2 co-writer\/director}{2{2 Peter}{3{2 Jackson}{2 ''s}}}}{3{2 expanded}{2 vision}}}{2{2 of}{2{2{2 J.R.R.}{2{2 Tolkien}{2 ''s}}}{2 Middle-earth}}}}}}}}}{2 .}}}'
    ),
    (
        9,
        '{3{2 I}{3{3{2 found}{3{2{2{2 The}{2 Ring}}{2 moderately}}{3{2{3{3 absorbing}{2 ,}}{2 largely}}{3{2 for}{3{3{3{2 its}{3{3 elegantly}{3{3 colorful}{2 look}}}}{2 and}}{2 sound}}}}}}{2 .}}}'
    ),
    (
        12,
        '{2{2 Snipes}{1{0{2{2 relies}{1{2 too}{2 much}}}{1{2 on}{2{3{2 a}{1{2{2 scorchingly}{2 plotted}}{2{3 dramatic}{2 scenario}}}}{2{2 for}{3{2 its}{2{2 own}{3 good}}}}}}}{2 .}}}'
    ),
    (
        12,
        '{1{1{2 Where}{1{2{2 Tom}{2 Green}}{1{1{2 stages}{2{2 his}{2 gags}}}{1{2 as}{1{2 assaults}{2{2 on}{1{2{2 America}{2 ''s}}{2{2 knee-jerk}{2{2 moral}{2 sanctimony}}}}}}}}}}{1{2 ,}{1{2 Jackass}{1{1{1 lacks}{2{2 aspirations}{2{2 of}{2{2 social}{2 upheaval}}}}}{2 .}}}}}'
    ),
    (
        10,
        '{3{3{2{2{2 The}{2 cast}}{2 ...}}{3{3{3{3{2 keeps}{3{2 this}{3{4 pretty}{3 watchable}}}}{2 ,}}{2 and}}{3{2{2 casting}{2{2 Mick}{2 Jagger}}}{3{2 as}{2{2{2 director}{2{2 of}{2{2 the}{2{2 escort}{2 service}}}}}{3{2 was}{3 inspired}}}}}}}{2 .}}'
    ),
    (
        9,
        '{4{2 This}{4{4{2 is}{3{2 a}{4{3 superior}{2{1 horror}{2 flick}}}}}{2 .}}}'
    ),
    (
        9,
        '{1{1{2{2 A}{2{1 rambling}{2{2 ensemble}{2 piece}}}}{1{2 with}{1{1{1{1{2 loosely}{2 connected}}{2 characters}}{2 and}}{1{2 plots}{1{2 that}{1{1{2 never}{2 quite}}{2 gel}}}}}}}{2 .}}'
    ),
    (
        9,
        '{3{3{2{2{2 The}{2 lightest}}{2 ,}}{3{3{2 most}{3{2 breezy}{2 movie}}}{2{2 Steven}{2 Spielberg}}}}{2{2{2 has}{2{2 made}{2{2 in}{2{3{2 more}{2{2 than}{2 a}}}{2 decade}}}}}{2 .}}}'
    ),
    (
        11,
        '{2{2{2{2 Nair}{2 and}}{2{2 writer}{2{2 Laura}{2 Cahill}}}}{3{3{2 dare}{3{2 to}{2{2{2 build}{2{2 a}{2 movie}}}{3{2 around}{2{1{1{2 some}{1{0{2{1 flawed}{2{2 but}{2 rather}}}{0 unexceptional}}{2 women}}}{2 ,}}{2{2 emerging}{3{2 with}{3{3{2 a}{3{3 fine}{2{2 character}{2 study}}}}{3{2 that}{3{2 ''s}{3{1{1{2 short}{2{2 on}{2 plot}}}{2 but}}{4{3 rich}{2{2 in}{3{2{2 the}{2{2 tiny}{2 revelations}}}{2{2 of}{2{3 real}{3 life}}}}}}}}}}}}}}}}}{2 .}}}'
    ),
    (
        10,
        '{3{2 It}{3{3{2 ''s}{3{4 fun}{2 lite}}}{2 .}}}'
    ),
    (
        11,
        '{4{4{4{4{2{2 The}{2 film}}{4{2 is}{4{3 visually}{4 dazzling}}}}{2 ,}}{3{2{2 the}{2{2 depicted}{2 events}}}{4{3 dramatic}{3{2 ,}{4{3{3 funny}{2 and}}{4 poignant}}}}}}{2 .}}'
    ),
    (
        9,
        '{3{3 Delivers}{3{2{2 more}{3{2 than}{2{2{2 its}{2{2 fair}{3 share}}}{3{2 of}{3 saucy}}}}}{3{3 hilarity}{2 .}}}}'
    ),
    (
        12,
        '{2{2{2 The}{2 film}}{2{2{2 is}{2{2{2{2 about}{2{2 the}{2 relationships}}}{2{2 rather}{2 than}}}{2{2 about}{2{2 the}{2 outcome}}}}}{2 .}}}'
    ),
    (
        10,
        '{0{1{2{2{2{2{2 The}{2{2 Paradiso}{2 ''s}}}{1{1 rusted-out}{1 ruin}}}{2 and}}{1{2{3 ultimate}{1 collapse}}{2{2 during}{2{2{2 the}{2{2 film}{2 ''s}}}{2{2 final}{2{2{1 -LRB-}{2{2 restored}{3 -RRB-}}}{2 third}}}}}}}{2 ...}}{0{2 emotionally}{1{1{2 belittle}{4{2 a}{4{2 cinema}{3 classic}}}}{2 .}}}}'
    ),
    (
        9,
        '{2{2{2 Once}{2 again}}{1{2 ,}{0{0{1{2{1{2{2{2 the}{3 intelligence}}{2{2 of}{2{2 gay}{2 audiences}}}}{3{2 has}{1{2 been}{2{1 grossly}{3 underestimated}}}}}{2 ,}}{2 and}}{2{3{3{3{2 a}{3{3 meaty}{2 plot}}}{2 and}}{3{3 well-developed}{2 characters}}}{1{2{1{2 have}{1{2 been}{2{2 sacrificed}{2{2 for}{2 skin}}}}}{2 and}}{1{2 flash}{1{2 that}{1{2 barely}{1 fizzle}}}}}}}{2 .}}}}'
    ),
    (
        10,
        '{4{2 This}{4{4{4{3{2 makes}{2{2{2 Minority}{2 Report}}{3{2 necessary}{2{2 viewing}{2{2 for}{2{2 sci-fi}{3 fans}}}}}}}{2 ,}}{4{2 as}{4{2{2 the}{2 film}}{3{3{2 has}{3{2 some}{3{2 of}{3{2 the}{4{4 best}{2{3 special}{2 effects}}}}}}}{2 ever}}}}}{2 .}}}'
    ),
    (
        10,
        '{4{4{4{4 Awesome}{2 work}}{2 :}}{4{2{3 ineffable}{3{2 ,}{3{2{2{2 elusive}{2 ,}}{2 yet}}{2 inexplicably}}}}{4 powerful}}}'
    );


create table if not exists inverted_select_ds(
    id int ,
    threshold int,
    query_tree invertedtree
);

insert into inverted_select_ds select id, threshold, treearena_to_inverted_label_list(query_tree) from tree_select_ds;


create table if not exists sed_select_ds(
    id int ,
    threshold int,
    query_tree sedindex
);

insert into sed_select_ds select id, threshold, treearena_to_sed_index(query_tree) from tree_select_ds;

