int __cdecl evaluate_script(int a1, int a2)
{
  int *v2; // edi
  BOOL v3; // ebx
  BOOL v4; // edi
  int v5; // ebp
  bool v6; // zf
  BOOL v7; // ebx
  unsigned int i; // eax
  int v10; // ebp
  int v11; // edx
  __int16 v12; // di
  __int16 v13; // ax
  _BYTE *v14; // eax
  int v15; // ecx
  __int16 v16; // si
  const char *v17; // ebp
  __int16 v18; // di
  __int16 v19; // si
  char *v20; // eax
  char v21; // cl
  char *v22; // ebp
  char *v23; // ecx
  char v24; // al
  __int16 v25; // di
  const char *v26; // ebp
  const CHAR *v27; // eax
  char *v28; // ecx
  char v29; // dl
  BOOL v30; // eax
  char *v31; // eax
  char v32; // cl
  char *v33; // eax
  __int16 v35; // ax
  __int16 v36; // ax
  char *v37; // eax
  char *v38; // edx
  char v39; // cl
  char *v40; // ecx
  char *v41; // edx
  char v42; // al
  char *v43; // eax
  int v45; // eax
  __int16 v46; // ax
  int v47; // ecx
  __int16 v48; // ax
  char *v49; // eax
  char *v50; // edx
  char v51; // cl
  char *v52; // eax
  int v54; // eax
  int *v55; // ebx
  int *v56; // edi
  void *v57; // eax
  __int16 v58; // ax
  __int16 v59; // ax
  int v60; // ecx
  int v61; // edx
  unsigned int v62; // edx
  int v63; // eax
  BOOL v64; // ecx
  int v65; // edx
  char *v66; // eax
  char v67; // cl
  char *v68; // eax
  int v70; // ecx
  int v71; // eax
  int v72; // eax
  char *v73; // eax
  char v74; // cl
  const char *v75; // eax
  int v76; // ecx
  int v77; // eax
  double v78; // st6
  const CHAR *v79; // eax
  double v80; // st7
  char v81; // al
  char *v82; // eax
  char v83; // cl
  char *v84; // eax
  int v86; // eax
  int v87; // eax
  int v88; // ecx
  int v89; // eax
  char *v90; // eax
  char v91; // cl
  const char *v92; // eax
  int v93; // ecx
  int v94; // eax
  char v95; // cl
  char *v96; // eax
  int v98; // eax
  int v99; // ecx
  LONG v100; // edi
  const CHAR *v101; // eax
  char v102; // al
  int v103; // eax
  int v104; // ecx
  char *v105; // eax
  char v106; // cl
  char *v107; // eax
  bool v109; // zf
  __int16 v110; // ax
  __int16 v111; // ax
  int v112; // ecx
  int v113; // edx
  int v114; // esi
  char *v115; // edx
  char *v116; // eax
  char v117; // cl
  char *v118; // eax
  char v119; // cl
  char *v120; // eax
  int v122; // eax
  int *v123; // ebx
  int *v124; // edi
  void *v125; // eax
  int v126; // eax
  int v127; // eax
  int v128; // eax
  BOOL v129; // eax
  int v130; // ecx
  double v131; // st7
  int v132; // eax
  int v133; // ecx
  BOOL v134; // eax
  int v135; // ecx
  int v136; // edi
  int v137; // edx
  int v138; // ecx
  unsigned int v139; // eax
  BOOL v140; // edx
  int *v141; // eax
  BOOL v142; // ecx
  int *v143; // eax
  int v144; // eax
  int v145; // edi
  int v146; // edx
  int v147; // eax
  int *v148; // eax
  int v149; // ecx
  int v150; // eax
  int v151; // ecx
  int v152; // edx
  int *v153; // eax
  char *v154; // eax
  int v156; // eax
  int v157; // eax
  char *v158; // eax
  char *v159; // ebx
  char v160; // cl
  char *v161; // eax
  int v163; // eax
  char *v164; // eax
  char v165; // cl
  char *v166; // eax
  unsigned int v168; // eax
  int v169; // eax
  char *v170; // edx
  char *v171; // eax
  char v172; // cl
  BOOL v173; // ecx
  int v174; // edx
  int v175; // ecx
  int v176; // ebp
  int v177; // esi
  int v178; // edi
  char v179; // al
  int v180; // eax
  int v181; // esi
  int v182; // ebx
  char v183; // al
  int v184; // edx
  int v185; // edi
  char v186; // al
  char v187; // al
  int v188; // edx
  int v189; // eax
  BOOL v190; // ecx
  int v191; // eax
  int v192; // eax
  const char *v193; // eax
  const CHAR *v194; // eax
  char *v195; // eax
  char v196; // cl
  char *v197; // eax
  char *v199; // edx
  char *v200; // eax
  char v201; // cl
  int v202; // ecx
  int v203; // edx
  const char *v204; // eax
  const CHAR *v205; // eax
  int v206; // eax
  int v207; // ebx
  int *v208; // edi
  int v209; // eax
  int v210; // ecx
  unsigned int v211; // eax
  int v212; // eax
  int v213; // ebx
  int v214; // edi
  int v215; // edi
  char *v216; // eax
  char v217; // cl
  int v218; // esi
  int v219; // edi
  char *v220; // eax
  int v222; // eax
  int v223; // ebx
  int v224; // eax
  int v225; // ecx
  int v226; // edx
  int v227; // esi
  int v228; // edi
  int v229; // eax
  const char *v230; // eax
  const CHAR *v231; // eax
  __int16 v232; // ax
  int v233; // esi
  __int16 v234; // ax
  int v235; // ebp
  int v236; // edi
  int v237; // eax
  char v238; // cl
  char *v239; // eax
  char v240; // cl
  char *v241; // eax
  char *v243; // eax
  char v244; // cl
  int v245; // ebx
  int v246; // edi
  const char *v247; // eax
  const CHAR *v248; // eax
  int v249; // edi
  double v250; // st7
  long double v251; // st7
  int v252; // ecx
  int v253; // eax
  int v254; // esi
  int v255; // eax
  __int16 v256; // ax
  int v257; // eax
  __int16 v258; // cx
  __int16 v259; // cx
  __int16 v260; // ax
  __int16 v261; // dx
  __int16 v262; // di
  int v263; // eax
  int v264; // ecx
  const CHAR *v265; // eax
  char *v266; // ecx
  char v267; // al
  char *v268; // ecx
  char v269; // al
  const char *v270; // eax
  unsigned __int8 v271; // cl
  int v272; // eax
  char *v273; // eax
  int v275; // eax
  int v276; // ecx
  const char *v277; // edi
  int v278; // eax
  CHAR *v279; // eax
  CHAR v280; // dl
  int v281; // esi
  int v282; // ebx
  int v283; // ebp
  char *v284; // eax
  char *v285; // eax
  char *v286; // edx
  char v287; // cl
  char *v288; // ecx
  char v289; // al
  char *v290; // eax
  int v292; // eax
  int v293; // ecx
  int v294; // eax
  int v295; // eax
  unsigned int v296; // ecx
  int v297; // eax
  int v298; // edx
  _BOOL2 v299; // ax
  int v300; // ecx
  __int16 v301; // ax
  int v302; // edx
  __int16 v303; // ax
  const char *v304; // ebp
  const char *v305; // eax
  const CHAR *v306; // eax
  const CHAR *v307; // eax
  int v308; // edx
  char v309; // al
  BOOL v310; // eax
  unsigned int v311; // eax
  char *v312; // eax
  BOOL v313; // eax
  const CHAR *v314; // eax
  int v315; // ecx
  _DWORD *v316; // esi
  const char *v317; // eax
  const CHAR *v318; // eax
  int v319; // ebx
  int v320; // ecx
  int v321; // eax
  int v322; // esi
  int (__cdecl *v323)(int, int, int); // eax
  int v324; // ebx
  int v325; // edi
  int v326; // edi
  const CHAR *v327; // eax
  int v328; // eax
  int v329; // esi
  int instruction_ptr_saved; // ebp
  _WORD *v331; // edi
  __int16 v332; // cx
  const char *v333; // ebp
  const char *v334; // ecx
  char v335; // al
  unsigned int v336; // eax
  unsigned __int8 v337; // al
  int v338; // eax
  char *v339; // edx
  int v340; // eax
  char *v341; // eax
  char v342; // cl
  __int16 v343; // ax
  const CHAR *v344; // eax
  int v345; // ecx
  int v346; // eax
  int v347; // edx
  int *v348; // eax
  int v349; // edx
  int v350; // ecx
  int *v351; // eax
  int v352; // ecx
  const char *v353; // eax
  int v354; // eax
  int v355; // ecx
  int v356; // edx
  const char *v357; // eax
  const char *v358; // eax
  const char *v359; // eax
  const CHAR *v360; // eax
  const char *v361; // eax
  const CHAR *v362; // eax
  int v363; // ebp
  const char *v364; // eax
  int X; // [esp+8h] [ebp-5CCCh]
  int Xa; // [esp+8h] [ebp-5CCCh]
  int *Xb; // [esp+8h] [ebp-5CCCh]
  char *X_4; // [esp+Ch] [ebp-5CC8h]
  char *v369; // [esp+10h] [ebp-5CC4h]
  char *v370; // [esp+10h] [ebp-5CC4h]
  const CHAR *v371; // [esp+10h] [ebp-5CC4h]
  __int64 v372; // [esp+10h] [ebp-5CC4h]
  __int64 v373; // [esp+10h] [ebp-5CC4h]
  const CHAR *v374; // [esp+10h] [ebp-5CC4h]
  const CHAR *v375; // [esp+10h] [ebp-5CC4h]
  const CHAR *v376; // [esp+10h] [ebp-5CC4h]
  const CHAR *v377; // [esp+10h] [ebp-5CC4h]
  int v378; // [esp+14h] [ebp-5CC0h]
  int v379; // [esp+14h] [ebp-5CC0h]
  char *v380; // [esp+14h] [ebp-5CC0h]
  int v381; // [esp+18h] [ebp-5CBCh]
  HWND hWnd; // [esp+2Ch] [ebp-5CA8h]
  unsigned int v383; // [esp+30h] [ebp-5CA4h]
  double v384; // [esp+30h] [ebp-5CA4h]
  int v385; // [esp+30h] [ebp-5CA4h]
  int v386; // [esp+30h] [ebp-5CA4h]
  int v387; // [esp+30h] [ebp-5CA4h]
  int v388; // [esp+30h] [ebp-5CA4h]
  int v389; // [esp+30h] [ebp-5CA4h]
  char v390; // [esp+3Fh] [ebp-5C95h]
  __int16 v391; // [esp+40h] [ebp-5C94h]
  double v392; // [esp+40h] [ebp-5C94h]
  double v393; // [esp+40h] [ebp-5C94h]
  int v394; // [esp+40h] [ebp-5C94h]
  int v395; // [esp+40h] [ebp-5C94h]
  double v396; // [esp+48h] [ebp-5C8Ch]
  double v397; // [esp+48h] [ebp-5C8Ch]
  const char *v398; // [esp+48h] [ebp-5C8Ch]
  double v399; // [esp+48h] [ebp-5C8Ch]
  int v400; // [esp+48h] [ebp-5C8Ch]
  int v401; // [esp+48h] [ebp-5C8Ch]
  int v402; // [esp+54h] [ebp-5C80h]
  int v403; // [esp+54h] [ebp-5C80h]
  char v404[4]; // [esp+58h] [ebp-5C7Ch] BYREF
  int v405; // [esp+5Ch] [ebp-5C78h]
  int v406; // [esp+60h] [ebp-5C74h] BYREF
  int v407; // [esp+64h] [ebp-5C70h]
  int v408; // [esp+68h] [ebp-5C6Ch]
  int v409; // [esp+6Ch] [ebp-5C68h]
  int v410; // [esp+70h] [ebp-5C64h]
  int v411; // [esp+74h] [ebp-5C60h]
  int v412; // [esp+78h] [ebp-5C5Ch]
  int v413; // [esp+7Ch] [ebp-5C58h]
  int v414; // [esp+88h] [ebp-5C4Ch] BYREF
  int v415; // [esp+8Ch] [ebp-5C48h]
  int v416; // [esp+A0h] [ebp-5C34h] BYREF
  int v417; // [esp+A4h] [ebp-5C30h]
  char v418[20]; // [esp+A8h] [ebp-5C2Ch] BYREF
  int v419; // [esp+BCh] [ebp-5C18h] BYREF
  int v420; // [esp+C0h] [ebp-5C14h] BYREF
  int v421; // [esp+C4h] [ebp-5C10h]
  int v422; // [esp+C8h] [ebp-5C0Ch] BYREF
  int v423; // [esp+CCh] [ebp-5C08h] BYREF
  CHAR String2[268]; // [esp+D0h] [ebp-5C04h] BYREF
  char v425[20]; // [esp+1DCh] [ebp-5AF8h] BYREF
  RECT rcSrc1; // [esp+1F0h] [ebp-5AE4h] BYREF
  double Y[3]; // [esp+200h] [ebp-5AD4h] BYREF
  int v428; // [esp+218h] [ebp-5ABCh]
  char v429[20]; // [esp+21Ch] [ebp-5AB8h] BYREF
  CHAR SrcStr[64]; // [esp+230h] [ebp-5AA4h] BYREF
  CHAR DestStr[64]; // [esp+270h] [ebp-5A64h] BYREF
  CHAR v432[1023]; // [esp+2B0h] [ebp-5A24h] BYREF
  char v433; // [esp+6AFh] [ebp-5625h] BYREF
  char v434[256]; // [esp+6B0h] [ebp-5624h] BYREF
  char Str[256]; // [esp+7B0h] [ebp-5524h] BYREF
  char v436[1024]; // [esp+8B0h] [ebp-5424h] BYREF
  CHAR v437[1024]; // [esp+CB0h] [ebp-5024h] BYREF
  CHAR v438[1024]; // [esp+10B0h] [ebp-4C24h] BYREF
  CHAR v439[1024]; // [esp+14B0h] [ebp-4824h] BYREF
  char Buffer[1024]; // [esp+18B0h] [ebp-4424h] BYREF
  CHAR v441[1024]; // [esp+1CB0h] [ebp-4024h] BYREF
  char Src[15396]; // [esp+20B0h] [ebp-3C24h] BYREF

  v405 = dword_4E6BC0;
  hWnd = ::hWnd;
  if ( !dword_4F88F4 )
    return 0;
  if ( *(_DWORD *)(dword_92B210 + 3136) )
    return 1;
  if ( (dword_4FB008 || dword_4FB00C) && dword_4FB01C && dword_4FB02C && !sub_465D70() )
  {
    if ( dword_804618 && !dword_4FB048 && !dword_80462C )
    {
      dword_804624 += dword_4FB058 + 1;
      dword_80462C = 1;
    }
    dword_4FB014 = 0;
    dword_4FB01C = 0;
    dword_4FB040 = 0;
    dword_4FB03C = 0;
  }
  v2 = dword_4F891C;
  do
  {
    if ( *(v2 - 1) && *v2 && !sub_4352D0() )
      dword_61E524[31708 * *(v2 - 5) + 10 * *(v2 - 3)] = 1;
    v2 += 9;
  }
  while ( (int)v2 < (int)&dword_4F8964 );
  v3 = dword_92B0B4 && dword_92B0B4 < 3;
  v4 = dword_4F5AB0 && dword_4F5AB4 > 0;
  v5 = sub_42C6A0();
  if ( dword_4FB01C | dword_4FB040 | dword_4FB03C | dword_600518 | dword_5FB5A0 | dword_4FAFEC | dword_92B140 | dword_5FA8A4 | dword_4F6ED0 | dword_5FCA48 | dword_4FEE00 | dword_4FEE1C | v3 | v4 | sub_416F50() | v5
    || dword_4FEC7C
    || dword_92B154
    || dword_6DB528 && dword_6DA944 )
  {
    return 1;
  }
  if ( dword_4F8950 || dword_4F5110 || dword_4F50F0 && !dword_4F50F4 || dword_5F9070 && !dword_5F9074 )
    return 1;
  if ( !dword_80461C )
    goto LABEL_41;
  v6 = dword_4FAFE0 == 0;
  while ( 2 )
  {
    if ( !v6 )
      return 1;
LABEL_41:
    // Branch handling.
    v7 = 1;
    if ( dword_4F50F0 && dword_4F50F4 || dword_5F9070 && dword_5F9074 )
    {
      for ( i = 0; i < 0x1E; ++i )
      {
        if ( byte_4F3AEC[i] == *(_BYTE *)instruction_ptr )
          goto LABEL_51;
      }
      if ( !word_6D9910 )
        sub_42CE40(0);
      return 1;
    }
LABEL_51:
    v10 = instruction_ptr + 1;
    if ( *(_BYTE *)instruction_ptr == 1 )
    {
      v11 = v421;
      do
      {
        v12 = heap[*(__int16 *)(v10 + 1)];
        if ( (*(_BYTE *)v10 & 0x10) != 0 )
          v13 = heap[*(__int16 *)(v10 + 3)];
        else
          v13 = *(_WORD *)(v10 + 3);
        switch ( *(_BYTE *)v10 & 0xF )
        {
          case 1:
            v11 = v12 >= v13;
            break;
          case 2:
            v11 = v12 <= v13;
            break;
          case 3:
            v11 = v12 == v13;
            break;
          case 4:
            v11 = v12 != v13;
            break;
          case 5:
            v11 = v12 > v13;
            break;
          case 6:
            v11 = v12 < v13;
            break;
          default:
            break;
        }
        v14 = (_BYTE *)(instruction_ptr + 11);
        instruction_ptr += 11;
        if ( !v11 )
        {
          v14 += *(_DWORD *)(v10 + 5);
          instruction_ptr = (int)v14;
        }
        v10 = (int)(v14 + 1);
      }
      while ( *v14 == 1 );
      v421 = v11;
    }
    v15 = *(unsigned __int8 *)instruction_ptr - 2;
    v390 = *(_BYTE *)instruction_ptr;
    switch ( *(_BYTE *)instruction_ptr )
    {
      case 2:
        instruction_ptr += 3;
        dword_5FB5A0 = 1;
        v328 = *(char *)v10;
        v329 = 0;
        dword_5FB5AC = v328;
        dword_5FB5B0 = 0;
        if ( v328 <= 0 )
          goto LABEL_1027;
        instruction_ptr_saved = instruction_ptr;
        v331 = &unk_5FB5B4;
        while ( 1 )
        {
          v332 = *(_WORD *)instruction_ptr_saved;
          v333 = (const char *)(instruction_ptr_saved + 2);
          v331[8] = v332;
          *((_DWORD *)v331 + 2) = 0;
          instruction_ptr = (int)v333;
          v334 = v333;
          do
          {
            v335 = *v334;
            v334[(char *)(v331 + 4) - v333 + 18] = *v334;
            ++v334;
          }
          while ( v335 );
          v336 = (unsigned int)&v333[strlen(v333) + 1];
          instruction_ptr = v336;
          if ( *(_BYTE *)v336 )
            *(_DWORD *)v331 = heap[*(__int16 *)(v336 + 1)] != 0;
          else
            *(_DWORD *)v331 = *(_WORD *)(v336 + 1) != 0;
          if ( *(_DWORD *)v331 )
            ++dword_5FB5B0;
          instruction_ptr_saved = v336 + 3;
          instruction_ptr = instruction_ptr_saved;
          v337 = *(_BYTE *)(v336 + 3);
          *((_BYTE *)v331 + 4) = *(_BYTE *)instruction_ptr_saved;
          v338 = v337 - 3;
          v339 = (char *)(instruction_ptr_saved + 1);
          if ( v338 )
          {
            v340 = v338 - 3;
            if ( v340 )
            {
              if ( v340 == 1 )
              {
                v341 = (char *)(instruction_ptr_saved + 1);
                do
                {
                  v342 = *v341;
                  v341[(char *)v331 - v339 + 1050] = *v341;
                  ++v341;
                }
                while ( v342 );
                instruction_ptr += strlen((const char *)(instruction_ptr_saved + 1)) + 2;
                instruction_ptr_saved = instruction_ptr;
              }
              goto LABEL_1026;
            }
            *((_DWORD *)v331 + 3) = *(_DWORD *)v339;
            instruction_ptr_saved += 6;
          }
          else
          {
            *((_BYTE *)v331 + 5) = *v339;
            *((_DWORD *)v331 + 5) = *(__int16 *)(instruction_ptr_saved + 2);
            if ( *(_BYTE *)(instruction_ptr_saved + 4) )
              v343 = heap[*(__int16 *)(instruction_ptr_saved + 5)];
            else
              v343 = *(_WORD *)(instruction_ptr_saved + 5);
            v331[12] = v343;
            instruction_ptr_saved += 8;
          }
          instruction_ptr = instruction_ptr_saved;
LABEL_1026:
          ++v329;
          v331 += 656;
          if ( v329 >= dword_5FB5AC )
          {
LABEL_1027:
            dword_5FB5A8 = 1;
            dword_5FB5A4 = 1;
            if ( dword_804618 )
            {
              v344 = (const CHAR *)sub_45F0F0(17);
              sub_4650C0(hWnd, 0, v344, &byte_4C81F8);
              dword_804624 = 0;
              dword_804628 = 0;
            }
            if ( !dword_5FB5B0 )
            {
              dword_5FB5A0 = 0;
              goto LABEL_252;
            }
            if ( *(_DWORD *)&dword_92A3E0 )
              dword_4F50D8 = 0;
            dword_4FB004 = !*(_DWORD *)&dword_92A3DC && (dword_4FB008 || dword_4FB00C);
            dword_4FB00C = 0;
            dword_4FB008 = 0;
            dword_4FAFE0 = 0;
            sub_4370F0();
            sub_45EEB0(350);
            sub_413E40();
            goto LABEL_253;
          }
        }
      case 3:
        v254 = word_6D988C;
        v400 = word_6D98EC;
        v7 = 0;
        v388 = word_6D9906;
        switch ( *(_BYTE *)v10 )
        {
          case 0:
            memset(heap, 0, 0x7D0u);
            v255 = 10;
            dword_4FB00C = 0;
            dword_4FB008 = 0;
            dword_92B160 = 0;
            if ( word_6D98EC != 10 )
              v255 = *(_DWORD *)&dword_92A474;
            sub_42FD20(v255);
            dword_4FB060 = word_6D98EC;
            break;
          case 1:
            if ( *(_BYTE *)(v10 + 3) )
              v256 = heap[*(__int16 *)(v10 + 4)];
            else
              v256 = *(_WORD *)(v10 + 4);
            heap[*(__int16 *)(v10 + 1)] = v256;
            if ( *(_WORD *)(v10 + 1) == 978 )
            {
              v257 = 10;
              if ( word_6D98EC != 10 )
                v257 = *(_DWORD *)&dword_92A474;
              sub_42FD20(v257);
              dword_4FB060 = word_6D98EC;
              sub_42CE40(0);
            }
            break;
          case 2:
            if ( *(_BYTE *)(v10 + 3) )
              v258 = heap[*(__int16 *)(v10 + 4)];
            else
              v258 = *(_WORD *)(v10 + 4);
            heap[*(__int16 *)(v10 + 1)] += v258;
            break;
          case 3:
            if ( *(_BYTE *)(v10 + 3) )
              v259 = heap[*(__int16 *)(v10 + 4)];
            else
              v259 = *(_WORD *)(v10 + 4);
            heap[*(__int16 *)(v10 + 1)] -= v259;
            break;
          case 4:
            if ( *(_BYTE *)(v10 + 3) )
              v260 = heap[heap[*(__int16 *)(v10 + 4)]];
            else
              v260 = heap[*(__int16 *)(v10 + 4)];
            heap[*(__int16 *)(v10 + 1)] = v260;
            break;
          case 5:
            if ( *(_BYTE *)(v10 + 3) )
              v261 = heap[*(__int16 *)(v10 + 4)];
            else
              v261 = *(_WORD *)(v10 + 4);
            if ( v261 )
              heap[*(__int16 *)(v10 + 1)] %= v261;
            break;
          case 6:
            if ( *(_BYTE *)(v10 + 3) )
              v262 = heap[*(__int16 *)(v10 + 4)];
            else
              v262 = *(_WORD *)(v10 + 4);
            if ( v262 )
              heap[*(__int16 *)(v10 + 1)] = rand() % v262;
            break;
          default:
            break;
        }
        if ( v254 != word_6D988C )
        {
          dword_6DB404 = 0;
          dword_4FB030 = 1;
        }
        if ( v400 != word_6D98EC )
        {
          dword_6DB404 = 0;
          dword_4FB030 = 1;
        }
        if ( v388 != word_6D9906 )
          sub_45B6E0();
        instruction_ptr += 8;
        goto LABEL_253;
      case 4:
        if ( word_6D9908 )
        {
          dword_4FEC50 = 1;
          if ( v405 == 1 )
          {
            if ( sub_422060(-1, 3) )
            {
              sub_45EEB0(250);
              ++instruction_ptr;
              goto LABEL_253;
            }
          }
          else
          {
            sub_422060(1, 3);
          }
          ++instruction_ptr;
        }
        else if ( !dword_600518 )
        {
LABEL_972:
          sub_435360();
        }
        goto LABEL_253;
      case 5:
        if ( *(_BYTE *)v10 && (dword_4FB008 || dword_4FB00C) )
        {
          dword_4FAFF0 = 0;
          if ( !word_6D9910 )
            sub_42CE40(0);
          v7 = 0;
          instruction_ptr += 3;
        }
        else
        {
          if ( dword_4FAFF0 || dword_4FAFF4 )
          {
            dword_4FAFEC = 1;
            if ( !word_6D9910 )
              sub_42CE40(0);
          }
          instruction_ptr += 3;
        }
        goto LABEL_253;
      case 6:
        v7 = 0;
        instruction_ptr = (int)dword_4F88F4 + *(_DWORD *)v10;
        goto LABEL_253;
      case 7:
        v266 = &String2[-v10];
        do
        {
          v267 = *(_BYTE *)v10;
          v266[v10] = *(_BYTE *)v10;
          ++v10;
        }
        while ( v267 );
        if ( !sub_409BB0() )
          goto LABEL_805;
        goto LABEL_197;
      case 8:
        instruction_ptr += 2;
        goto LABEL_253;
      case 9:
        instruction_ptr += strlen((const char *)v10) + 2;
        v268 = &String2[-v10];
        do
        {
          v269 = *(_BYTE *)v10;
          v268[v10] = *(_BYTE *)v10;
          ++v10;
        }
        while ( v269 );
        if ( sub_409DF0(String2) )
          goto LABEL_197;
        if ( !dword_92B1FC )
          goto LABEL_1084;
        goto LABEL_1074;
      case 0xA:
        if ( sub_409E80() )
          goto LABEL_197;
        if ( !dword_92B1FC )
        {
          v270 = (const char *)sub_45F0F0(13);
          sprintf(v432, v270);
          v371 = (const CHAR *)sub_45F0F0(17);
          sub_4650C0(hWnd, 0x10u, v371, v432);
        }
        goto LABEL_1074;
      case 0xB:
        dword_4FAFF0 = 1;
        dword_4FAFF4 = 0;
        dword_4FAFF8 = timeGetTime();
        v271 = *(_BYTE *)v10;
        instruction_ptr += 3;
        dword_4FAFFC = dword_4FAFF8 + 100 * v271;
        v7 = 0;
        goto LABEL_253;
      case 0xC:
        v272 = *(__int16 *)v10;
        instruction_ptr += 4;
        v7 = 0;
        heap[v272] = dword_4FAFF4 != 0;
        goto LABEL_253;
      case 0xD:
        v7 = 0;
        v263 = 0;
        if ( *(__int16 *)(v10 + 2) <= 0 )
          goto LABEL_800;
        while ( 1 )
        {
          v264 = *(__int16 *)v10 + v263;
          if ( v264 < 0 )
            break;
          heap[v264] = *(_WORD *)(v10 + 4);
          if ( ++v263 >= *(__int16 *)(v10 + 2) )
          {
            instruction_ptr += 8;
            goto LABEL_253;
          }
        }
        v265 = (const CHAR *)sub_45F0F0(17);
        sub_4650C0(hWnd, 0, v265, &byte_4C8150, byte_4F88E0);
LABEL_800:
        instruction_ptr += 8;
        goto LABEL_253;
      case 0xE:
        v6 = *(_BYTE *)v10 == 0;
        dword_4FB00C = 0;
        v7 = 0;
        instruction_ptr += 3;
        dword_4FB008 = 0;
        dword_4F50D8 = 0;
        dword_4FEC80 = !v6;
        goto LABEL_253;
      case 0x21:
        dword_4FEC90 = *(char *)(v10 + 3);
        if ( !dword_4FEC88 )
          goto LABEL_733;
        v239 = (char *)(v10 + 10);
        do
        {
          v240 = *v239;
          String2[(_DWORD)v239 - 10 - v10] = *v239;
          ++v239;
        }
        while ( v240 );
        if ( !strstr(String2, SubStr) )
        {
          v241 = (char *)&v423 + 3;
          while ( *++v241 )
            ;
          strcpy(v241, ".ogg");
        }
        if ( !_stricmp(byte_4FEC94, String2) )
        {
          sub_462FF0(0, (int)((double)dword_4FEC90 * (double)dword_92A3BC * 0.1));
          sub_462FF0(1, (int)((double)(10 - dword_4FEC90) * (double)dword_92A3BC * 0.1));
          dword_4FEC8C = *(__int16 *)(v10 + 4);
          instruction_ptr += strlen((const char *)(v10 + 10)) + 12;
          v7 = 0;
          goto LABEL_253;
        }
        dword_4FEC88 = 0;
LABEL_733:
        v243 = (char *)(v10 + 10);
        do
        {
          v244 = *v243;
          byte_4FEC94[(_DWORD)v243 - 10 - v10] = *v243;
          ++v243;
        }
        while ( v244 );
        v6 = dword_92A43C == 0;
        strcat(byte_4FEC94, ".ogg");
        v245 = *(__int16 *)(v10 + 4);
        dword_4FEC8C = v245;
        v246 = *(unsigned __int16 *)(v10 + 1);
        if ( !v6 )
          goto LABEL_741;
        if ( sub_462D90(
               0,
               (int)((double)dword_4FEC90 * (double)dword_92A3BC * 0.1),
               *(_BYTE *)v10 != 0,
               v246,
               v245,
               *(_DWORD *)(v10 + 6)) )
        {
          sub_462D90(
            1,
            (int)((double)(10 - dword_4FEC90) * (double)dword_92A3BC * 0.1),
            *(_BYTE *)v10 != 0,
            v246,
            dword_4FEC8C,
            *(_DWORD *)(v10 + 6));
LABEL_741:
          dword_4FEC88 = 1;
        }
        else
        {
          if ( !dword_92B1FC )
          {
            v247 = (const char *)sub_45F0F0(15);
            sprintf(v441, v247, byte_4FEC94);
            v248 = (const CHAR *)sub_45F0F0(17);
            sub_4650C0(hWnd, 0x10u, v248, v441);
          }
          dword_4FEC88 = 0;
        }
        v62 = instruction_ptr + strlen((const char *)(v10 + 10)) + 12;
        goto LABEL_251;
      case 0x22:
        v249 = *(unsigned __int16 *)(v10 + 1);
        if ( *(_BYTE *)v10 && *(_WORD *)(v10 + 1) )
        {
          sub_462FB0(*(unsigned __int16 *)(v10 + 1));
          sub_462FB0(v249);
          instruction_ptr += 5;
          dword_4FEC88 = 0;
          v7 = 0;
        }
        else
        {
          sub_462F70(0);
          sub_462F70(1);
          instruction_ptr += 5;
          dword_4FEC88 = 0;
          v7 = 0;
        }
        goto LABEL_253;
      case 0x23:
      case 0x27:
        if ( dword_4F8918[9 * (*(_BYTE *)instruction_ptr != 35)] && dword_4F891C[9 * (*(_BYTE *)instruction_ptr != 35)] )
          dword_61E524[31708 * dword_4F8908[9 * (*(_BYTE *)instruction_ptr != 35)]
                     + 10 * dword_4F8910[9 * (*(_BYTE *)instruction_ptr != 35)]] = 1;
        v216 = (char *)(v10 + 9);
        v398 = (const char *)(v10 + 9);
        do
        {
          v217 = *v216;
          String2[(_DWORD)v216 - 9 - v10] = *v216;
          ++v216;
        }
        while ( v217 );
        v387 = *(__int16 *)(v10 + 5);
        v218 = sub_43D410(String2);
        v403 = v218;
        v219 = sub_460C00(String2, (int)dword_5F8738, (int)&v423, (int)&v422);
        if ( !strstr(String2, SubStr) )
        {
          v220 = (char *)&v423 + 3;
          while ( *++v220 )
            ;
          strcpy(v220, ".ogg");
        }
        v222 = *(char *)(v10 + 7);
        v223 = 9 * (v390 != 35);
        dword_4F8920[v223] = 0;
        dword_4F890C[v223] = v222;
        dword_4F8914[v223] = v218;
        if ( v222 == 101 )
        {
          v224 = v219 != 9999 ? v219 : 0;
          if ( v219 != 9999 )
          {
            v225 = v422;
            v226 = v423;
            dword_4F8920[9 * (v390 != 35)] = 1;
            dword_4F8924[9 * (v390 != 35)] = v225;
            dword_4F8928[9 * (v390 != 35)] = v226;
          }
        }
        else
        {
          v224 = 10 * v222;
        }
        v227 = *(char *)(v10 + 8);
        v228 = *(_DWORD *)&dword_92A458 == 0 ? v224 : 0;
        sub_421080(v387, v228, v227);
        if ( !dword_4FB008 && !dword_4FB00C )
        {
          sub_434F50();
          if ( !sub_43A3B0(v403) )
          {
            v229 = sub_43A370(v403);
            if ( sub_463130(String2, 1, v229, 0, 0, 0, 0, v228, v227) )
            {
              dword_4F8918[9 * (v390 != 35)] = 1;
            }
            else if ( !dword_92B1FC )
            {
              v230 = (const char *)sub_45F0F0(15);
              sprintf(v439, v230, String2);
              v231 = (const CHAR *)sub_45F0F0(17);
              sub_4650C0(hWnd, 0x10u, v231, v439);
            }
            if ( (*(_BYTE *)v10 & 1) != 0 )
              v232 = heap[*(__int16 *)(v10 + 1)];
            else
              v232 = *(_WORD *)(v10 + 1);
            v233 = v232;
            if ( (*(_BYTE *)v10 & 2) != 0 )
              v234 = heap[*(__int16 *)(v10 + 3)];
            else
              v234 = *(_WORD *)(v10 + 3);
            v235 = v234;
            if ( v233 != -1 )
            {
              v236 = 126832 * v233 + 40 * v234;
              *(int *)((char *)dword_61E524 + v236) = 0;
              *(int *)((char *)dword_61E51C + v236) = 1;
              *(int *)((char *)dword_61E52C + v236) = timeGetTime();
              dword_4F8908[9 * (v390 != 35)] = v233;
              dword_4F8910[9 * (v390 != 35)] = v235;
              dword_4F891C[9 * (v390 != 35)] = 1;
            }
          }
        }
        instruction_ptr += strlen(v398) + 11;
        v237 = 0;
        do
        {
          v238 = String2[v237];
          byte_4FEE44[v237++] = v238;
        }
        while ( v238 );
        goto LABEL_252;
      case 0x24:
        sub_434F50();
        instruction_ptr += 2;
        v7 = 0;
        goto LABEL_253;
      case 0x25:
        dword_6DB428[13 * *(char *)v10] = 0;
        dword_6DB430[13 * *(char *)v10] = 0;
        dword_6DB42C[13 * *(char *)v10] = 0;
        dword_6DB420[13 * *(char *)v10] = 0;
        sub_4633B0(*(char *)v10);
        v195 = (char *)(v10 + 11);
        do
        {
          v196 = *v195;
          String2[(_DWORD)v195 - 11 - v10] = *v195;
          ++v195;
        }
        while ( v196 );
        if ( !strstr(String2, SubStr) )
        {
          v197 = (char *)&v423 + 3;
          while ( *++v197 )
            ;
          strcpy(v197, ".ogg");
        }
        dword_6DB420[13 * *(char *)v10] = 1;
        v199 = String2;
        v200 = (char *)&unk_6DB440 + 52 * *(char *)v10;
        do
        {
          v201 = *v199;
          *v200++ = *v199++;
        }
        while ( v201 );
        dword_6DB428[13 * *(char *)v10] = *(_BYTE *)(v10 + 1) == 0xFF;
        dword_6DB434[13 * *(char *)v10] = *(char *)(v10 + 6);
        dword_6DB424[13 * *(char *)v10] = *(unsigned __int16 *)(v10 + 7);
        dword_6DB438[13 * *(char *)v10] = 10 * *(char *)(v10 + 9);
        dword_6DB43C[13 * *(char *)v10] = *(char *)(v10 + 10);
        v386 = *(_BYTE *)(v10 + 3) != 0;
        if ( !*(_WORD *)(v10 + 4) )
          v386 = 0;
        if ( !dword_92A440 && (!dword_4FB008 && !dword_4FB00C || dword_6DB428[13 * *(char *)v10]) )
        {
          v202 = *(char *)(v10 + 1);
          if ( v202 >= 1 )
            --v202;
          if ( *(_DWORD *)&dword_92A458 )
            v203 = 0;
          else
            v203 = dword_6DB438[13 * *(char *)v10];
          if ( !sub_463130(
                  String2,
                  0,
                  (int)((double)dword_6DB434[13 * *(char *)v10] * (double)dword_92A3C0 * 0.1),
                  v202,
                  v386,
                  *(unsigned __int16 *)(v10 + 4),
                  dword_6DB424[13 * *(char *)v10],
                  v203,
                  dword_6DB43C[13 * *(char *)v10])
            && !dword_92B1FC )
          {
            v204 = (const char *)sub_45F0F0(15);
            sprintf(v437, v204, String2);
            v205 = (const CHAR *)sub_45F0F0(17);
            sub_4650C0(hWnd, 0x10u, v205, v437);
          }
        }
        instruction_ptr += strlen((const char *)(v10 + 11)) + 13;
        if ( dword_92A440 )
        {
          dword_6DB42C[13 * *(char *)v10] = 0;
          v7 = 0;
        }
        else
        {
          dword_6DB42C[13 * *(char *)v10] = *(_BYTE *)(v10 + 2) != 0;
          v206 = 13 * *(char *)v10;
          v7 = dword_6DB42C[v206] != 0;
          if ( dword_6DB42C[v206] )
          {
LABEL_346:
            if ( !word_6D9910 )
              sub_42CE40(0);
          }
        }
        goto LABEL_253;
      case 0x26:
        if ( *(_BYTE *)v10 == 0xFF )
        {
          v207 = 0;
          v208 = dword_6DB430;
          do
          {
            sub_4633B0(v207);
            *(v208 - 2) = 0;
            *v208 = 0;
            *(v208 - 1) = 0;
            *(v208 - 4) = 0;
            v208 += 13;
            ++v207;
          }
          while ( (int)v208 < (int)&dword_6DB534 );
          instruction_ptr += 3;
          v7 = 0;
        }
        else
        {
          sub_4633B0(*(char *)v10);
          dword_6DB428[13 * *(char *)v10] = 0;
          dword_6DB430[13 * *(char *)v10] = 0;
          dword_6DB42C[13 * *(char *)v10] = 0;
          v209 = *(char *)v10;
          instruction_ptr += 3;
          dword_6DB420[13 * v209] = 0;
          v7 = 0;
        }
        goto LABEL_253;
      case 0x28:
        dword_6DB434[13 * *(char *)v10] = *(char *)(v10 + 1);
        if ( dword_92A440 || (v213 = *(char *)v10, !*((_DWORD *)dword_804634 + 3)) || v213 >= 10 )
        {
          instruction_ptr += 6;
          v7 = 0;
        }
        else
        {
          sub_4638D0((int)(0.1 * ((double)dword_6DB434[13 * v213] * (double)dword_92A3C0)));
          instruction_ptr += 6;
          v7 = 0;
        }
        goto LABEL_253;
      case 0x29:
        v210 = *((_DWORD *)dword_804634 + 3);
        v211 = *(char *)v10;
        if ( v210 && v211 <= 9 && *(_DWORD *)(v210 + 4 * v211 + 16) )
          sub_463C60(*(unsigned __int16 *)(v10 + 1));
        dword_6DB428[13 * *(char *)v10] = 0;
        dword_6DB430[13 * *(char *)v10] = 0;
        dword_6DB42C[13 * *(char *)v10] = 0;
        v212 = *(char *)v10;
        instruction_ptr += 5;
        dword_6DB420[13 * v212] = 0;
        v7 = 0;
        goto LABEL_253;
      case 0x30:
        dword_4FEC90 = *(char *)v10;
        if ( !dword_92A43C )
        {
          sub_462FF0(0, (int)(0.1 * ((double)dword_4FEC90 * (double)dword_92A3BC)));
          sub_462FF0(1, (int)((double)(10 - dword_4FEC90) * (double)dword_92A3BC * 0.1));
        }
        instruction_ptr += 5;
        v7 = 0;
        goto LABEL_253;
      case 0x31:
        if ( !dword_92A440 )
        {
          v214 = 13 * *(char *)v10;
          if ( dword_6DB420[v214] )
          {
            if ( *((_DWORD *)dword_804634 + 3) && *(char *)v10 < 10 && sub_463920() )
              dword_6DB42C[v214] = 1;
          }
        }
        instruction_ptr += 3;
        goto LABEL_346;
      case 0x32:
        if ( !dword_92A440 )
        {
          v215 = 13 * *(char *)v10;
          if ( dword_6DB420[v215] )
          {
            if ( *((_DWORD *)dword_804634 + 3) && *(char *)v10 < 10 && sub_463920() )
              dword_6DB430[v215] = 1;
          }
        }
        instruction_ptr += 3;
        goto LABEL_346;
      case 0x33:
        if ( sub_4630D0() )
        {
          v250 = sub_4630A0();
          v399 = modf(v250, Y);
          v251 = Y[0];
          heap[*(__int16 *)v10] = (int)(0.01666666666666667 * Y[0]);
          heap[*(__int16 *)(v10 + 2)] = (int)fmod(v251, 60.0);
          v252 = *(__int16 *)(v10 + 4);
          instruction_ptr += 8;
          heap[v252] = (int)(v399 * 1000.0);
        }
        else
        {
          heap[*(__int16 *)v10] = -1;
          heap[*(__int16 *)(v10 + 2)] = -1;
          v253 = *(__int16 *)(v10 + 4);
          instruction_ptr += 8;
          heap[v253] = -1;
        }
        v7 = 0;
        goto LABEL_253;
      case 0x41:
        if ( dword_80461C && dword_4FAFE0 )
          return 1;
        memcpy_0(Src, &dword_4FB000, 0x3C20u);
        dword_80462C = 0;
        if ( (dword_804624 >= 15 || sub_436FC0()) && dword_92B618 != 3 )
        {
          dword_4FB030 = 1;
          dword_804624 = 0;
          dword_804628 = 0;
        }
        Count = 0;
        dword_4FB020 = 0;
        dword_4FB044 = 0;
        dword_4FB048 = 0;
        dword_4FB03C = 0;
        dword_4FB034 = 0;
        dword_4FB038 = 1;
        dword_4FB080 = 0;
        dword_4FB05C = 100;
        dword_4FB050 = word_6D98EE != 0;
        v16 = *(_WORD *)v10;
        dword_4FB064 = *(char *)(v10 + 2);
        dword_92B618 = dword_4FB064;
        dword_4FB070 = *(char *)(v10 + 3);
        memset(&Source, 0, 0x400u);
        v17 = (const char *)(v10 + 4);
        strcpy(&Source, v17);
        dword_4FB02C = 0;
        dword_4FB084 = 0;
        dword_4FB088 = 0;
        dword_4FB08C = 0;
        dword_4FB090 = 0;
        dword_4FB818 = -1;
        dword_4FB81C = 0;
        memset(&dword_4FB820, 0, 0x1000u);
        memset(&word_4FC820, 0, 0x400u);
        memset(&dword_4FCC20, 0, 0x1000u);
        memset(&word_4FDC20, 0, 0x800u);
        if ( !dword_804628 || !dword_804618 )
          sub_43E610();
        dword_4FB02C = 0;
        dword_4F50DC = 0;
        dword_4F50E0 = strlen(&Source);
        dword_4FB01C = 1;
        dword_4FB014 = 1;
        instruction_ptr += strlen(v17) + 6;
        sub_42AC00(0);
        do_something_perhaps_validation_with_string(&Source, 0, 0);
        sub_42BFA0(&Source);
        sub_43CB20();
        if ( dword_804618 && dword_804624 + sub_421780(&dword_4FB094) >= 15 )
        {
          dword_4FB030 = 1;
          dword_804624 = 0;
          dword_804628 = 0;
        }
        if ( byte_92B614 != 35 && byte_92B614 != 39 )
          dword_6DB540[1188 * dword_6DB530] = 0;
        if ( dword_4FB064 == 3 )
        {
          if ( !dword_4FB038 )
            sub_421780(&dword_4FB094);
        }
        else if ( !dword_4FB038 )
        {
          sub_421080(0, 0, 0);
          goto LABEL_90;
        }
        if ( dword_804618 )
          ++dword_804630;
LABEL_90:
        v18 = dword_4FB00C;
        dword_4FB024 = sub_42C8B0(v16, 1);
        if ( !*(_DWORD *)&dword_92A3EC || !sub_42C8B0(v16, 0) || (dword_4FB00C = 1, dword_4FB010) )
          dword_4FB00C = 0;
        if ( *(_DWORD *)&dword_92A3D4 )
        {
          if ( dword_4FB008 )
          {
            if ( !a2 && !sub_42C8B0(v16, 1) )
            {
              dword_4FB008 = 0;
              if ( byte_92B614 == 35 || byte_92B614 == 39 )
                sub_424B70(0);
            }
          }
        }
        if ( !v18 && dword_4FB00C )
          sub_434F50();
        dword_4FB010 = 0;
        sub_42C820(0);
        sub_42CDD0();
        if ( word_6D990A > 0 )
        {
          dword_4FB074 = 1;
          dword_4FB07C = timeGetTime();
          dword_4FB078 = word_6D990A;
        }
        if ( dword_4FB038 )
        {
          if ( dword_4FB034 && dword_4FB064 == 3 )
          {
            memcpy_0(&dword_4FB000, Src, 0x3C20u);
            dword_4FB064 = 3;
            dword_4FB034 = 1;
            dword_4FB01C = 1;
            dword_4FB014 = 1;
            if ( dword_804624 )
              --dword_804624;
          }
          else if ( dword_4FB03C )
          {
            rcSrc1.left = 18;
            rcSrc1.top = 36;
            rcSrc1.right = 800;
            rcSrc1.bottom = 600;
            if ( sub_4658E0(&rcSrc1, 0, 0, -1, 0, 0, 0, 0, 1) )
              dword_4FB040 = 1;
            dword_4FB03C = 0;
            dword_4FB01C = 0;
            dword_4FB02C = 1;
            dword_4FB044 = 1;
          }
        }
        goto LABEL_253;
      case 0x42:
        if ( dword_80461C && dword_4FAFE0 )
          return 1;
        memcpy_0(Src, &dword_4FB000, 0x3C20u);
        dword_80462C = 0;
        if ( (dword_804624 >= 15 || sub_436FC0()) && dword_92B618 != 3 )
        {
          dword_4FB030 = 1;
          dword_804624 = 0;
          dword_804628 = 0;
        }
        Count = 0;
        dword_4FB020 = 1;
        dword_4FB044 = 0;
        dword_4FB048 = 0;
        dword_4FB040 = 0;
        dword_4FB03C = 0;
        dword_4FB034 = 0;
        dword_4FB038 = 1;
        dword_4FB080 = 0;
        dword_4FB05C = 100;
        dword_4FB050 = word_6D98EE != 0;
        v19 = *(_WORD *)v10;
        dword_4FB064 = *(char *)(v10 + 2);
        dword_92B618 = dword_4FB064;
        dword_4FB06C = *(char *)(v10 + 3);
        dword_4FB070 = *(char *)(v10 + 4);
        memset(byte_4FE420, 0, sizeof(byte_4FE420));
        v20 = (char *)(v10 + 5);
        do
        {
          v21 = *v20;
          byte_4FE420[(_DWORD)v20 - 5 - v10] = *v20;
          ++v20;
        }
        while ( v21 );
        memset(&Source, 0, 0x400u);
        v22 = (char *)(strlen(byte_4FE420) + v10 + 6);
        v23 = (char *)(&Source - v22);
        do
        {
          v24 = *v22;
          v22[(_DWORD)v23] = *v22;
          ++v22;
        }
        while ( v24 );
        dword_4FB02C = 0;
        dword_4FB084 = 0;
        dword_4FB088 = 0;
        dword_4FB08C = 0;
        dword_4FB090 = 0;
        dword_4FB818 = -1;
        dword_4FB81C = 0;
        memset(&dword_4FB820, 0, 0x1000u);
        memset(&word_4FC820, 0, 0x400u);
        memset(&dword_4FCC20, 0, 0x1000u);
        memset(&word_4FDC20, 0, 0x800u);
        if ( !dword_804628 || !dword_804618 )
          sub_43E610();
        dword_4FB02C = 0;
        dword_4F50DC = 0;
        dword_4F50E0 = strlen(&Source);
        dword_4FB01C = 1;
        dword_4FB014 = 1;
        instruction_ptr += strlen(byte_4FE420) + strlen(&Source) + 8;
        sub_42AC00(0);
        do_something_perhaps_validation_with_string(&Source, 0, 0);
        sub_42BFA0(&Source);
        sub_43CB20();
        if ( dword_804618 && dword_804624 + sub_421780(&dword_4FB094) >= 15 )
        {
          dword_4FB030 = 1;
          dword_804624 = 0;
          dword_804628 = 0;
        }
        if ( byte_92B614 != 35 && byte_92B614 != 39 )
          dword_6DB540[1188 * dword_6DB530] = 0;
        if ( dword_4FB064 == 3 )
        {
          if ( !dword_4FB038 )
            sub_421780(&dword_4FB094);
        }
        else if ( !dword_4FB038 )
        {
          sub_421080(0, 0, 0);
          goto LABEL_141;
        }
        if ( dword_804618 )
          ++dword_804630;
LABEL_141:
        v25 = dword_4FB00C;
        dword_4FB024 = sub_42C8B0(v19, 1);
        if ( !*(_DWORD *)&dword_92A3EC || !sub_42C8B0(v19, 0) || (dword_4FB00C = 1, dword_4FB010) )
          dword_4FB00C = 0;
        if ( *(_DWORD *)&dword_92A3D4 )
        {
          if ( dword_4FB008 )
          {
            if ( !a2 && !sub_42C8B0(v19, 1) )
            {
              dword_4FB008 = 0;
              if ( byte_92B614 == 35 || byte_92B614 == 39 )
                sub_424B70(0);
            }
          }
        }
        if ( !v25 && dword_4FB00C )
          sub_434F50();
        dword_4FB010 = 0;
        sub_42C820(0);
        sub_42CDD0();
        if ( word_6D990A > 0 )
        {
          dword_4FB074 = 1;
          dword_4FB07C = timeGetTime();
          dword_4FB078 = word_6D990A;
        }
        if ( dword_4FB038 && (dword_4FB034 || dword_4FB03C) && dword_4FB064 == 3 )
        {
          memcpy_0(&dword_4FB000, Src, 0x3C20u);
          dword_4FB064 = 3;
          dword_4FB01C = 1;
          dword_4FB014 = 1;
        }
        goto LABEL_253;
      case 0x43:
        v31 = (char *)(v10 + 6);
        do
        {
          v32 = *v31;
          String2[(_DWORD)v31 - 6 - v10] = *v31;
          ++v31;
        }
        while ( v32 );
        v33 = (char *)&v423 + 3;
        while ( *++v33 )
          ;
        strcpy(v33, ".anm");
        if ( (*(_BYTE *)(v10 + 5) & 1) != 0 )
          v35 = heap[*(__int16 *)(v10 + 1)];
        else
          v35 = *(_WORD *)(v10 + 1);
        dword_61DB9C[31708 * *(char *)v10] = v35;
        if ( (*(_BYTE *)(v10 + 5) & 2) != 0 )
          v36 = heap[*(__int16 *)(v10 + 3)];
        else
          v36 = *(_WORD *)(v10 + 3);
        dword_61DBA0[31708 * *(char *)v10] = v36;
        if ( dword_5F8740[84 * *(char *)v10] )
        {
          sub_416CB0(*(char *)v10, 0);
          dword_5F8740[84 * *(char *)v10] = 0;
        }
        dword_61E508[31708 * *(char *)v10] = 0;
        dword_5F8740[84 * *(char *)v10] = 1;
        if ( *(_BYTE *)v10 )
        {
          sub_43B620(v418);
        }
        else
        {
          v37 = (char *)sub_43B400(v381);
          v38 = v418;
          do
          {
            v39 = *v37;
            *v38++ = *v37++;
          }
          while ( v39 );
        }
        if ( sub_416A30(*(char *)v10) )
        {
          if ( !*(_BYTE *)v10 )
          {
            v40 = (char *)&unk_600538;
            v41 = String2;
            do
            {
              v42 = *v40;
              *v41++ = *v40++;
            }
            while ( v42 );
            v43 = (char *)&v423 + 3;
            while ( *++v43 )
              ;
            strcpy(v43, ".wip");
            v45 = sub_43B400(&v414);
            sub_467D80(v45, X, v404, 1, 0);
            if ( (*(_BYTE *)(v10 + 5) & 1) != 0 )
              v46 = heap[*(__int16 *)(v10 + 1)];
            else
              v46 = *(_WORD *)(v10 + 1);
            v47 = v46;
            dword_5F874C[0] = v46;
            if ( (*(_BYTE *)(v10 + 5) & 2) != 0 )
              v48 = heap[*(__int16 *)(v10 + 3)];
            else
              v48 = *(_WORD *)(v10 + 3);
            dword_5F8750[0] = v48;
            dword_5F8754[0] = v47 + 800;
            dword_5F8758[0] = v48 + 600;
            dword_5F875C[0] = 0;
            dword_5F8760[0] = 0;
            dword_5F8764[0] = 800;
            dword_5F8768[0] = 600;
            dword_5F873C[0] = 1;
            dword_5F8738[0] = 1;
            goto LABEL_244;
          }
          if ( dword_5F876C[84 * *(char *)v10] )
          {
            free((void *)dword_5F876C[84 * *(char *)v10]);
            dword_5F876C[84 * *(char *)v10] = 0;
          }
          v49 = (char *)&unk_600538 + 126832 * *(char *)v10;
          v50 = String2;
          do
          {
            v51 = *v49;
            *v50++ = *v49++;
          }
          while ( v51 );
          v52 = (char *)&v423 + 3;
          while ( *++v52 )
            ;
          strcpy(v52, ".msk");
          sub_43B620(v418);
          sub_467D80(v418, &v414, v404, 1, 1);
          v54 = 84 * *(char *)v10;
          v55 = &dword_5F8770[v54];
          v56 = &dword_5F876C[v54];
          v57 = malloc(v414 * v415 + 7);
          *v56 = (int)v57;
          if ( !v57 || !sub_465220(v55) )
          {
LABEL_1078:
            sub_4253B0(hWnd, 0);
            sub_435360();
            return 0;
          }
          if ( sub_467FF0(&v414, dword_5F8770[84 * *(char *)v10], 1, 0) )
          {
            sub_44A550(*(char *)v10 + 1125, v414, v415);
            if ( (*(_BYTE *)(v10 + 5) & 1) != 0 )
              v58 = heap[*(__int16 *)(v10 + 1)];
            else
              v58 = *(_WORD *)(v10 + 1);
            dword_5F875C[84 * *(char *)v10] = v58;
            if ( (*(_BYTE *)(v10 + 5) & 2) != 0 )
              v59 = heap[*(__int16 *)(v10 + 3)];
            else
              v59 = *(_WORD *)(v10 + 3);
            dword_5F8760[84 * *(char *)v10] = v59;
            v60 = v414;
            dword_5F8764[84 * *(char *)v10] = v414 + dword_5F875C[84 * *(char *)v10];
            v61 = v415;
            dword_5F8768[84 * *(char *)v10] = v415 + dword_5F8760[84 * *(char *)v10];
            dword_5F874C[84 * *(char *)v10] = 0;
            dword_5F8750[84 * *(char *)v10] = 0;
            dword_5F8758[84 * *(char *)v10] = v61;
            dword_5F8754[84 * *(char *)v10] = v60;
            dword_5F873C[84 * *(char *)v10] = 1;
            dword_5F8738[84 * *(char *)v10] = 1;
            dword_4FA304[*(char *)v10] = 0;
LABEL_244:
            dword_5F87E0[84 * *(char *)v10] = 0;
            dword_5F8790[84 * *(char *)v10] = 0;
            if ( *(_BYTE *)v10 )
            {
              if ( dword_5F8810[84 * *(char *)v10] )
                free((void *)dword_5F8810[84 * *(char *)v10]);
              dword_5F8810[84 * *(char *)v10] = 0;
              sub_44A6E0();
            }
            dbl_5F87D0[42 * *(char *)v10] = 1.0;
            dbl_5F87C8[42 * *(char *)v10] = 1.0;
            dbl_5F87D8[42 * *(char *)v10] = 0.0;
            if ( !*(_BYTE *)v10 )
            {
              dword_4E6D74 = 400;
              dword_4E6D78 = 300;
            }
            dword_5F8788[84 * *(char *)v10] = 0;
            v62 = instruction_ptr + strlen((const char *)(v10 + 6)) + 8;
LABEL_251:
            instruction_ptr = v62;
            goto LABEL_252;
          }
          if ( dword_92B1FC )
            goto LABEL_1074;
        }
        else if ( dword_92B1FC )
        {
          goto LABEL_1074;
        }
        goto LABEL_1084;
      case 0x44:
        dword_61E524[31708 * *(char *)v10 + 10 * *(char *)(v10 + 1)] = 1;
        v63 = 5 * *(char *)(v10 + 1);
        v64 = *(_BYTE *)(v10 + 2) != 0;
        v65 = 31708 * *(char *)v10;
        instruction_ptr += 5;
        dword_61E520[2 * v63 + v65] = v64;
        if ( !dword_61E520[31708 * *(char *)v10 + 10 * *(char *)(v10 + 1)] )
          v7 = 0;
        goto LABEL_253;
      case 0x45:
        dword_61E518[31708 * *(char *)v10 + 10 * *(char *)(v10 + 1)] = 0;
        dword_61E524[31708 * *(char *)v10 + 10 * *(char *)(v10 + 1)] = 0;
        dword_61E51C[31708 * *(char *)v10 + 10 * *(char *)(v10 + 1)] = 1;
        dword_61E52C[31708 * *(char *)v10 + 10 * *(char *)(v10 + 1)] = timeGetTime();
        v7 = *(_BYTE *)(v10 + 2) != 0;
        instruction_ptr += 5;
        goto LABEL_253;
      case 0x46:
        if ( dword_5F8740[0] )
        {
          sub_416CB0(0, 0);
          dword_61E508[0] = 0;
          dword_5F8740[0] = 0;
          v7 = 0;
        }
        v66 = (char *)(v10 + 9);
        do
        {
          v67 = *v66;
          String2[(_DWORD)v66 - 9 - v10] = *v66;
          ++v66;
        }
        while ( v67 );
        v68 = (char *)&v423 + 3;
        while ( *++v68 )
          ;
        strcpy(v68, ".wip");
        if ( (*(_BYTE *)(v10 + 8) & 1) != 0 )
          v70 = heap[*(unsigned __int16 *)v10];
        else
          v70 = *(unsigned __int16 *)v10;
        dword_5F874C[0] = v70;
        v71 = *(unsigned __int16 *)(v10 + 2);
        if ( (*(_BYTE *)(v10 + 8) & 2) != 0 )
          v71 = heap[v71];
        dword_5F8750[0] = v71;
        dword_5F8754[0] = v70 + 800;
        dword_5F8758[0] = v71 + 600;
        dword_5F875C[0] = 0;
        dword_5F8760[0] = 0;
        dword_5F8764[0] = 800;
        dword_5F8768[0] = 600;
        if ( v7 )
        {
          v72 = _stricmp((const char *)(v10 + 9), "BLACK");
          if ( !v72 && dword_5F8748 == *(_DWORD *)(v10 + 4) )
          {
            LOBYTE(v72) = dword_5F8790[0] == 0;
            if ( v72 )
              goto LABEL_280;
          }
        }
        dword_5F8748 = *(_DWORD *)(v10 + 4);
        v73 = (char *)(v10 + 9);
        do
        {
          v74 = *v73;
          aBlack[(_DWORD)v73 - 9 - v10] = *v73;
          ++v73;
        }
        while ( v74 );
        v75 = (const char *)sub_43B400(String2);
        if ( !sub_4011D0(14, dword_5F8748, v75, v369, 0) )
        {
LABEL_1081:
          if ( !dword_92B1FC )
          {
            v380 = String2;
            goto LABEL_1073;
          }
          goto LABEL_1074;
        }
        dword_5F8790[0] = 0;
        dword_5F8738[0] = 1;
        sub_466C70();
        sub_466C70();
LABEL_280:
        dbl_5F87D0[0] = 1.0;
        dbl_5F87C8[0] = 1.0;
        dword_4E6D78 = 300;
        dword_4E6D80 = 300;
        v7 = 0;
        dbl_5F87D8[0] = 0.0;
        dword_4E6D74 = 400;
        dword_4E6D7C = 400;
        dword_5F873C[0] = 1;
        dword_5F87E0[0] = 0;
        instruction_ptr += strlen((const char *)(v10 + 9)) + 11;
        goto LABEL_253;
      case 0x47:
        v81 = *(_BYTE *)v10;
        if ( *(_BYTE *)v10 == 2 )
        {
          v409 = (int)&unk_4F3200;
          v408 = (int)&unk_4F3200;
          v406 = dword_600524;
          v407 = 1;
          v410 = 0;
          v411 = -2;
          sub_44AF20();
          instruction_ptr += 3;
          dword_4FAFE0 = 0;
          dword_5004E8 = 0;
          dword_4FAFD8 = 0;
          dword_92B188 = 1;
          v7 = 0;
        }
        else
        {
          instruction_ptr += 3;
          dword_5F8774 = 0;
          v7 = 0;
          dword_5F873C[0] = v81 != 0;
        }
        goto LABEL_253;
      case 0x48:
        if ( dword_5F8890[84 * *(char *)v10] )
        {
          sub_416CB0(*(char *)v10 + 1, 0);
          *((_DWORD *)&unk_63D478 + 31708 * *(char *)v10) = 0;
          dword_5F8890[84 * *(char *)v10] = 0;
          v7 = 0;
        }
        v105 = (char *)(v10 + 11);
        do
        {
          v106 = *v105;
          String2[(_DWORD)v105 - 11 - v10] = *v105;
          ++v105;
        }
        while ( v106 );
        v107 = (char *)&v423 + 3;
        while ( *++v107 )
          ;
        strcpy(v107, ".wip");
        if ( !v7 )
          goto LABEL_367;
        if ( !_stricmp((const char *)(v10 + 11), (const char *)&unk_5F88C8 + 336 * *(char *)v10)
          && dword_5F8898[84 * *(char *)v10] == *(_DWORD *)(v10 + 5) )
        {
          v7 = dword_5F88E0[84 * *(char *)v10] == 0;
          if ( !dword_5F88E0[84 * *(char *)v10] )
          {
            v7 = 0;
            v109 = *(_BYTE *)(v10 + 10)
                 ? dword_5F88D8[84 * *(char *)v10] == 0
                 : dword_5F88D8[84 * *(char *)v10] == dword_92B170[*(char *)v10];
            LOBYTE(v7) = v109;
            if ( v7 )
              goto LABEL_369;
          }
        }
        else
        {
          v7 = 0;
        }
LABEL_367:
        if ( dword_5F88BC[84 * *(char *)v10] )
        {
          free((void *)dword_5F88BC[84 * *(char *)v10]);
          dword_5F88BC[84 * *(char *)v10] = 0;
        }
LABEL_369:
        sub_43B620(v418);
        sub_467D80(v418, &v414, v404, 1, 0);
        if ( (*(_BYTE *)(v10 + 9) & 1) != 0 )
          v110 = heap[*(unsigned __int16 *)(v10 + 1)];
        else
          v110 = *(_WORD *)(v10 + 1);
        dword_5F88AC[84 * *(char *)v10] = v110;
        if ( (*(_BYTE *)(v10 + 9) & 2) != 0 )
          v111 = heap[*(unsigned __int16 *)(v10 + 3)];
        else
          v111 = *(_WORD *)(v10 + 3);
        dword_5F88B0[84 * *(char *)v10] = v111;
        v112 = v414;
        dword_5F88B4[84 * *(char *)v10] = v414 + dword_5F88AC[84 * *(char *)v10];
        v113 = v415;
        dword_5F88B8[84 * *(char *)v10] = v415 + dword_5F88B0[84 * *(char *)v10];
        dword_5F889C[84 * *(char *)v10] = 0;
        dword_5F88A0[84 * *(char *)v10] = 0;
        dword_5F88A8[84 * *(char *)v10] = v113;
        dword_5F88A4[84 * *(char *)v10] = v112;
        if ( *(_BYTE *)(v10 + 10) )
          v114 = 0;
        else
          v114 = dword_92B170[*(char *)v10];
        if ( v7 )
          goto LABEL_390;
        dword_5F8898[84 * *(char *)v10] = *(_DWORD *)(v10 + 5);
        v115 = (char *)(v10 + 11);
        v116 = (char *)&unk_5F88C8 + 336 * *(char *)v10;
        do
        {
          v117 = *v115;
          *v116++ = *v115++;
        }
        while ( v117 );
        if ( !sub_4011D0(dword_4F31E8[*(char *)v10], dword_5F8898[84 * *(char *)v10], v418, String2, v114) )
        {
LABEL_1083:
          if ( dword_92B1FC )
            goto LABEL_1074;
          goto LABEL_1084;
        }
        v118 = (char *)(v10 + 11);
        do
        {
          v119 = *v118;
          String2[(_DWORD)v118 - 11 - v10] = *v118;
          ++v118;
        }
        while ( v119 );
        v120 = (char *)&v423 + 3;
        while ( *++v120 )
          ;
        strcpy(v120, ".msk");
        sub_467D80(v418, &v414, v404, 1, 1);
        v122 = 84 * *(char *)v10;
        v123 = &dword_5F88C0[v122];
        v124 = &dword_5F88BC[v122];
        v125 = malloc(v414 * v415 + 7);
        *v124 = (int)v125;
        if ( !v125 || !sub_465220(v123) )
          goto LABEL_1078;
        if ( sub_467FF0(&v414, dword_5F88C0[84 * *(char *)v10], 1, v114) )
        {
          sub_44A550(*(char *)v10 + 1126, v414, v415);
          dword_5F88E0[84 * *(char *)v10] = 0;
          dword_5F8888[84 * *(char *)v10] = 1;
LABEL_390:
          dword_5F888C[84 * *(char *)v10] = 1;
          dword_5F8930[84 * *(char *)v10] = 0;
          if ( dword_5F8960[84 * *(char *)v10] )
            free((void *)dword_5F8960[84 * *(char *)v10]);
          dword_5F8960[84 * *(char *)v10] = 0;
          sub_44A6E0();
          dbl_5F8920[42 * *(char *)v10] = 1.0;
          dbl_5F8918[42 * *(char *)v10] = 1.0;
          dbl_5F8928[42 * *(char *)v10] = 0.0;
          dword_5F88D8[84 * *(char *)v10] = v114;
          v7 = 0;
          dword_4FA308[*(char *)v10] = 0;
          instruction_ptr += strlen((const char *)(v10 + 11)) + 13;
LABEL_253:
          if ( v390 != 39 )
            byte_92B614 = v390;
          v6 = !v7;
          continue;
        }
LABEL_1085:
        if ( !dword_92B1FC )
        {
          v380 = String2;
LABEL_1073:
          v357 = (const char *)sub_45F0F0(15);
          sprintf(v432, v357, v380);
          v375 = (const CHAR *)sub_45F0F0(17);
          sub_4650C0(hWnd, 0x10u, v375, v432);
        }
LABEL_1074:
        sub_435360();
        return 0;
      case 0x49:
        v134 = *(_BYTE *)(v10 + 1) != 0;
        v135 = 84 * *(char *)v10;
        instruction_ptr += 4;
        dword_5F888C[v135] = v134;
        v7 = 0;
        goto LABEL_253;
      case 0x4A:
        sub_416E10();
        sub_4170D0(0);
        dword_4FA304[0] = 1;
        dword_600518 = 1;
        dword_60051C = 0;
        if ( dword_92B0B8 )
        {
          v150 = *(char *)v10;
        }
        else
        {
          LOBYTE(v150) = *(_BYTE *)v10;
          if ( !*(_BYTE *)v10 || dword_4FB008 || dword_4FB00C )
            v150 = 0;
          else
            v150 = (char)v150;
        }
        dword_600520 = v150;
        v151 = *(unsigned __int16 *)(v10 + 1);
        dword_600530 = v151;
        dword_600534 = *(unsigned __int16 *)(v10 + 3);
        if ( v150 && !v151 )
          dword_600520 = 0;
        sub_412B60();
        instruction_ptr += 7;
        v152 = 0;
        if ( !word_6D9910 )
        {
          if ( dword_600520 )
            sub_42CE40(0);
          else
            sub_42CE40(1);
        }
        v153 = dword_5F8738;
        do
        {
          *v153 = v152;
          v153 += 84;
        }
        while ( (int)v153 < (int)&dword_5F9068 );
        if ( *((_DWORD *)dword_92B108 + 1062) != v152 )
        {
          word_6D9910 = 0;
          dword_4FAFE0 = v152;
          dword_4FAFD8 = v152;
        }
        goto LABEL_253;
      case 0x4B:
        v138 = *(char *)v10;
        if ( v138 <= 100 )
        {
          v139 = 336 * v138;
          dword_5F87A0[84 * v138] = 0;
          dword_5F879C[84 * v138] = 0;
        }
        else
        {
          v138 -= 100;
          v139 = 336 * v138;
          dword_5F87A0[v139 / 4] = 0;
          dword_5F879C[v139 / 4] = 1;
        }
        dword_5F8798[v139 / 4] = 1;
        word_5F87A4[v139 / 2] = *(_WORD *)(v10 + 1);
        word_5F87A6[v139 / 2] = *(_WORD *)(v10 + 3);
        dword_5F87B8[v139 / 4] = *(_DWORD *)(v10 + 5);
        dword_5F8794[v139 / 4] = *(__int16 *)(v10 + 9);
        dword_5F87BC[v139 / 4] = *(_DWORD *)(v10 + 11);
        dword_5F87C0[v139 / 4] = *(_DWORD *)(v10 + 15);
        if ( v138 )
        {
          dword_5F87A8[v139 / 4] = dword_5F875C[v139 / 4];
          dword_5F87AC[v139 / 4] = dword_5F8760[v139 / 4];
          dword_5F87B0[v139 / 4] = dword_5F8764[v139 / 4];
          dword_5F87B4[v139 / 4] = dword_5F8768[v139 / 4];
        }
        else
        {
          dword_5F87A8[0] = dword_5F874C[0];
          dword_5F87AC[0] = dword_5F8750[0];
          dword_5F87B0[0] = dword_5F8754[0];
          dword_5F87B4[0] = dword_5F8758[0];
        }
        instruction_ptr += 21;
        v7 = 0;
        if ( !dword_5F87B8[v139 / 4] )
          dword_5F8798[v139 / 4] = 0;
        if ( !*(_BYTE *)v10 )
          dword_5F873C[0] = 1;
        goto LABEL_253;
      case 0x4C:
        v140 = *(_DWORD *)(v10 + 3) != 0;
        dword_4F50F8 = *(_DWORD *)(v10 + 3);
        dword_4F50F0 = 1;
        dword_4F50FC = a1;
        dword_4F50EC = v140;
        dword_4F50F4 = *(_BYTE *)v10 != 0;
        dword_4F50E8 = *(char *)(v10 + 1);
        if ( dword_4FB008 || dword_4FB00C || !dword_4FEC80 && *(_DWORD *)&dword_92A450 )
          sub_439EF0(1);
        instruction_ptr += 9;
        if ( dword_5F8794[0] <= 0 )
        {
          if ( dword_91CA1C )
            sub_44A480(dword_5F8794[0] != -2 ? 0 : 0xFFFFFF);
        }
        else
        {
          v407 = dword_600524;
          v409 = (int)&unk_4F3ABC;
          v408 = (int)&unk_4F3ABC;
          v406 = 2015;
          v410 = 0;
          v411 = -2;
          sub_44AF20();
        }
        if ( !word_6D9910 )
          sub_42CE40(0);
        v141 = dword_5F8738;
        do
        {
          *v141 = 0;
          v141 += 84;
        }
        while ( (int)v141 < (int)&dword_5F9068 );
        goto LABEL_253;
      case 0x4D:
        if ( (dword_4FB008 || dword_4FB00C || !dword_4FEC80 && *(_DWORD *)&dword_92A450) && *(_BYTE *)(v10 + 1) != 0xFF )
        {
          if ( dword_4F5AB0 )
          {
            dword_4F5AB0 = 0;
            *(_DWORD *)&dword_92A488 = dword_92AE9C;
            dword_92A48C = dword_92AEA0;
            dword_4FB030 = 1;
          }
LABEL_566:
          v7 = 0;
          goto LABEL_567;
        }
        *(_DWORD *)&dword_92A488 = dword_92AE9C;
        dword_92A48C = dword_92AEA0;
        dword_4F5AB0 = *(char *)v10;
        dword_4F5AB4 = *(char *)(v10 + 1);
        dword_4F5ABC = *(__int16 *)(v10 + 2);
        dword_4F5AC0 = *(__int16 *)(v10 + 4);
        dword_4F5ACC = *(__int16 *)(v10 + 6);
        dword_4F5AC4 = *(__int16 *)(v10 + 8);
        dword_4F5AC8 = *(__int16 *)(v10 + 10);
        dword_4F5AB8 = 0;
        dword_4F5AD0 = 0;
        if ( dword_4F5AB0 == 5 )
        {
          memset(dword_4F5AD4, 0, sizeof(dword_4F5AD4));
          v175 = dword_4F5ABC;
          if ( dword_4F5ABC )
          {
            if ( dword_4F5ABC > 256 )
            {
              v175 = 256;
              dword_4F5ABC = 256;
            }
            v176 = 0;
            v177 = 256 / v175;
            do
            {
              v178 = v176 + rand() % v177;
              if ( v178 > 255 )
                v178 = 255;
              v176 += v177;
              dword_4F5AD4[v178] = rand() % 255;
            }
            while ( v176 < 256 );
          }
        }
        if ( dword_4F5AB4 <= 0 && (dword_4F5AB0 != 5 || dword_4F5AB4 != -2) )
          goto LABEL_566;
        if ( dword_4F5ACC )
          v7 = 0;
        if ( !word_6D9910 )
        {
          sub_42CE40(0);
          instruction_ptr += 14;
          goto LABEL_253;
        }
LABEL_567:
        instruction_ptr += 14;
        goto LABEL_253;
      case 0x4E:
        v179 = *(_BYTE *)v10;
        if ( *(_BYTE *)v10 == 9 || v179 == 10 )
        {
          memset(dword_5FE100, 0, 0x2418u);
          dword_5FE0C8 = 0;
          if ( *(_BYTE *)v10 == 10 )
          {
            dword_5FE120 = word_6D98A2;
            dword_5FE124 = word_6D98A4;
            dword_5FE118[0] = word_6D98A2;
            dword_5FE11C[0] = word_6D98A4;
            dword_5FE14C = word_6D98A8;
            dword_5FE150 = word_6D98AA;
            dword_5FE144 = word_6D98A8;
            dword_5FE148 = word_6D98AA;
            dword_5FE178 = word_6D98AE;
            dword_5FE17C = word_6D98B0;
            dword_5FE170 = word_6D98AE;
            dword_5FE174 = word_6D98B0;
            dword_5FE1A4 = word_6D98B4;
            dword_5FE1A8 = word_6D98B6;
            dword_5FE19C = word_6D98B4;
            dword_5FE1A0 = word_6D98B6;
          }
        }
        else if ( (dword_5FE0B8 || *(_BYTE *)(v10 + 2))
               && dword_5FE0BC != v179
               && ((unsigned int)(dword_5FE0BC - 3) > 2 || v179 < 3 || v179 > 5) )
        {
          memset(dword_5FE100, 0, 0x2418u);
          if ( *(_BYTE *)(v10 + 2) )
          {
            v407 = 11;
            v408 = 11;
            v409 = 8;
            v410 = 8;
            v180 = 0;
            v406 = 17;
            v411 = 7;
            v181 = 0;
            v385 = 0;
            v394 = 0;
            v402 = 0;
            while ( 1 )
            {
              v182 = 0;
              if ( *(&v406 + v180 + *(char *)(v10 + 1) % 2) > 0 )
                break;
LABEL_616:
              v394 += 8;
              v385 += 4;
              v180 = v402 + 2;
              v181 += 70;
              v402 += 2;
              if ( v402 >= 6 )
                goto LABEL_620;
            }
            while ( 1 )
            {
              v183 = *(_BYTE *)v10;
              if ( *(_BYTE *)v10 == 6 && v182 > v394 || (v183 == 14 || v183 == 15 || v183 == 16) && v182 > v385 )
                goto LABEL_616;
              if ( v183 < 3 || v183 == 6 || v183 == 11 )
              {
                v185 = 11 * (v182 + v181);
                v184 = rand() % 800;
              }
              else if ( v183 == 14 || v183 == 15 || v183 == 16 )
              {
                v185 = 11 * (v182 + v181);
                v184 = rand() % 800 + 400;
              }
              else if ( v183 <= 6 )
              {
                v184 = rand() % 900;
                v185 = 11 * (v182 + v181);
                dword_5FE118[v185] = v184;
                if ( *(_BYTE *)v10 != 4 )
                  v184 -= 100;
              }
              else
              {
                v184 = rand() % 2000;
                v185 = 11 * (v182 + v181);
                dword_5FE118[v185] = v184;
                if ( *(_BYTE *)v10 == 8 )
                  v184 -= 1200;
              }
              dword_5FE118[v185] = v184;
              dword_5FE108[v185] = 1;
              v186 = *(_BYTE *)v10;
              if ( *(_BYTE *)v10 == 2 )
                break;
              if ( v186 == 12 || v186 == 13 )
              {
                v188 = rand() % 600 - 259;
                goto LABEL_614;
              }
              dword_5FE10C[v185] = rand() % 240;
              v187 = *(_BYTE *)v10;
              if ( *(_BYTE *)v10 == 1 || v187 == 11 )
              {
                v188 = rand() % 600 - 14;
LABEL_614:
                dword_5FE11C[v185] = v188;
                goto LABEL_615;
              }
              if ( v187 != 14 && v187 != 15 && v187 != 16 )
              {
                if ( v187 == 6 )
                  v188 = rand() % 600 - 118;
                else
                  v188 = rand() % 600 - 28;
                goto LABEL_614;
              }
              v189 = rand();
              dword_5FE11C[v185] = 500;
              dword_5FE114[v185] = 0;
              dword_5FE104[v185] = 0;
              dword_5FE100[v185] = v189 % 6;
LABEL_615:
              ++v182;
              dword_5FE110[v185] = rand() % 3;
              if ( v182 >= *(&v406 + v402 + *(char *)(v10 + 1) % 2) )
                goto LABEL_616;
            }
            v188 = rand() % 600 - 298;
            goto LABEL_614;
          }
        }
LABEL_620:
        v190 = *(_BYTE *)v10 != 0;
        dword_5FE0B8 = v190;
        dword_5FE0BC = *(char *)v10;
        v191 = *(char *)(v10 + 1);
        dword_5FE0C0 = v191;
        dword_5FE0C4 = 1;
        if ( v191 > 1 )
        {
          dword_5FE0C4 = v191;
          dword_5FE0C0 = 1;
        }
        if ( !v190 )
          dword_4FB030 = 1;
        instruction_ptr += 5;
        v7 = 0;
        goto LABEL_253;
      case 0x4F:
        dword_61E51C[31708 * *(char *)v10 + 10 * *(char *)(v10 + 1)] = 0;
        dword_61E528[31708 * *(char *)v10 + 10 * *(char *)(v10 + 1)] = 0;
        dword_61DB98[31708 * *(char *)v10] = 1;
        v7 = *(_BYTE *)(v10 + 2) != 0;
        instruction_ptr += 5;
        goto LABEL_253;
      case 0x50:
        strcpy(String2, (const char *)v10);
        v273 = (char *)&v423 + 3;
        while ( *++v273 )
          ;
        strcpy(v273, ".tbl");
        v275 = sub_43B400(v381);
        if ( !sub_421940(v275) )
          goto LABEL_1081;
        v7 = 0;
        instruction_ptr += strlen((const char *)v10) + 2;
        goto LABEL_253;
      case 0x51:
        heap[*(__int16 *)v10] = word_4FFF16;
        v276 = *(__int16 *)(v10 + 2);
        instruction_ptr += 6;
        heap[v276] = word_4FFF18;
        goto LABEL_253;
      case 0x52:
        sub_421B70();
        v7 = 0;
        instruction_ptr += 3;
        goto LABEL_253;
      case 0x53:
        if ( (*(_BYTE *)v10 & 1) != 0 )
          v301 = heap[*(__int16 *)(v10 + 1)];
        else
          v301 = *(_WORD *)(v10 + 1);
        v302 = v301;
        if ( (*(_BYTE *)v10 & 2) != 0 )
          v303 = heap[*(__int16 *)(v10 + 3)];
        else
          v303 = *(_WORD *)(v10 + 3);
        v304 = (const char *)(v10 + 5);
        if ( sub_4254B0(v302, v303, v304) )
        {
          dword_8045C8 = 1;
        }
        else if ( !dword_92B1FC )
        {
          v305 = (const char *)sub_45F0F0(15);
          sprintf(v438, v305, v304);
          v306 = (const CHAR *)sub_45F0F0(17);
          sub_4650C0(hWnd, 0x10u, v306, v438);
        }
        dword_8045F4 = 0;
        instruction_ptr += strlen(v304) + 7;
        v7 = 0;
        goto LABEL_253;
      case 0x54:
        if ( dword_4F5ED8 )
          free(dword_4F5ED8);
        dword_4F5ED4 = 0;
        dword_4F5ED8 = 0;
        dword_4F5EDC = 0;
        dword_4F5EE0 = 0;
        dword_4F5EE4 = 0;
        dword_4F5EE8 = 0;
        dword_4F5EEC = 0;
        strcpy(String2, (const char *)v10);
        v154 = (char *)&v423 + 3;
        while ( *++v154 )
          ;
        strcpy(v154, ".msk");
        v156 = sub_43B400(1802726702);
        if ( !sub_401310(v156, String2, &dword_4F5ED8, &dword_4F5EDC, 0, 0) )
          goto LABEL_805;
        v7 = 0;
        dword_4F5ED4 = 1;
        strcpy((char *)&dword_4F5EE0, (const char *)v10);
        instruction_ptr += strlen((const char *)v10) + 2;
        goto LABEL_253;
      case 0x55:
        if ( dword_4F5ED8 )
          free(dword_4F5ED8);
        v7 = 0;
        instruction_ptr += 2;
        dword_4F5ED4 = 0;
        dword_4F5ED8 = 0;
        dword_4F5EDC = 0;
        dword_4F5EE0 = 0;
        dword_4F5EE4 = 0;
        dword_4F5EE8 = 0;
        dword_4F5EEC = 0;
        goto LABEL_253;
      case 0x56:
        sub_425600();
        instruction_ptr += 2;
        v7 = 0;
        goto LABEL_253;
      case 0x57:
        dword_8045F8 = 1;
        dword_804610 = a1;
        dword_804604 = *(__int16 *)v10;
        dword_804608 = *(__int16 *)(v10 + 2);
        dword_8045FC = dword_8045DC;
        dword_804600 = dword_8045E0;
        v137 = *(_DWORD *)(v10 + 4);
        v7 = 0;
        instruction_ptr += 10;
        dword_80460C = v137;
        goto LABEL_253;
      case 0x58:
        dword_61DBC4[31708 * *(char *)v10 + 6 * *(char *)(v10 + 1)] = *(__int16 *)(v10 + 3);
        dword_61DBC8[31708 * *(char *)v10 + 6 * *(char *)(v10 + 1)] = *(__int16 *)(v10 + 5);
        dword_61DB98[31708 * *(char *)v10] = 1;
        v7 = *(_BYTE *)(v10 + 2) != 0;
        instruction_ptr += 9;
        goto LABEL_253;
      case 0x59:
        sprintf(String2, "%s.wip", (const char *)v10);
        v192 = sub_43B400(v381);
        if ( !sub_41BA40(v192) && !dword_92B1FC )
        {
          v193 = (const char *)sub_45F0F0(15);
          sprintf(Buffer, v193, v10);
          v194 = (const CHAR *)sub_45F0F0(17);
          sub_4650C0(hWnd, 0x10u, v194, Buffer);
        }
        goto LABEL_196;
      case 0x60:
        if ( dword_5FE0E0 )
        {
          if ( dword_5FE0E8 )
            free(dword_5FE0E8);
          dword_5FE0E8 = 0;
          if ( dword_5FE0E0 )
            free(dword_5FE0E0);
          dword_5FE0E0 = 0;
          dword_5FE0F0 = 0;
          dword_5FE0F4 = 0;
          dword_5FE0F8 = 0;
          byte_5FE0FC = 0;
        }
        v7 = 0;
        instruction_ptr += 2;
        goto LABEL_253;
      case 0x61:
        sub_4170D0(0);
        v277 = (const char *)(v10 + 1);
        dword_4FEC84 = *(char *)v10;
        v378 = v10 + 1;
        if ( *(_DWORD *)&Data == 2 )
          sprintf(String2, "%s\\%s", &PathName, v378);
        else
          sprintf(String2, "%s\\%s", byte_92AD94, v378);
        if ( !sub_4587F0(String2, (int)Context, 0) )
        {
          if ( !dword_92B1FC )
          {
            v359 = (const char *)sub_45F0F0(15);
            sprintf(v432, v359, String2);
            v360 = (const CHAR *)sub_45F0F0(17);
            sub_4650C0(hWnd, 0x10u, v360, v432);
          }
          instruction_ptr += strlen((const char *)(v10 + 1)) + 3;
          return 0;
        }
        if ( !sub_459200() )
        {
          if ( !dword_92B1FC )
          {
            v361 = (const char *)sub_45F0F0(16);
            sprintf(v432, v361);
            v362 = (const CHAR *)sub_45F0F0(17);
            sub_4650C0(hWnd, 0x10u, v362, v432);
          }
          sub_459350();
          instruction_ptr += strlen((const char *)(v10 + 1)) + 3;
          return 0;
        }
        dword_92B0B4 = dword_4FEC84 + 1;
        instruction_ptr += strlen((const char *)(v10 + 1)) + 3;
        if ( dword_4FEC84 + 1 < 3 )
        {
          dword_4F50D8 = 0;
          dword_4FB00C = 0;
          dword_4FB008 = 0;
          dword_4FB014 = 0;
          sub_42CE40(0);
          goto LABEL_253;
        }
        if ( dword_92B0B4 == 3 )
        {
          if ( dword_5F8740[0] )
          {
            sub_416CB0(0, 0);
            dword_61E508[0] = 0;
            dword_5F8740[0] = 0;
          }
          sub_44A550(14, 800, 600);
          dword_5F873C[0] = 1;
          dword_5F87E0[0] = 0;
          v278 = 336 * *(char *)v10;
          dword_4E6D74 = 400;
          *(double *)(v278 + 6260688) = 1.0;
          dbl_5F87C8[0] = 1.0;
          dword_4E6D7C = 400;
          dbl_5F87D8[0] = 0.0;
          dword_4E6D78 = 300;
          dword_4E6D80 = 300;
          dword_5F8748 = 0;
          aBlack[0] = 0;
          v279 = (CHAR *)(v10 + 1);
          do
          {
            v280 = *v279;
            v279[MultiByteStr - v277] = *v279;
            ++v279;
          }
          while ( v280 );
          dword_5F8750[0] = 0;
          dword_5F874C[0] = 0;
          dword_5F8754[0] = 800;
          dword_5F8758[0] = 600;
          dword_5F875C[0] = 0;
          dword_5F8760[0] = 0;
          dword_5F8764[0] = 800;
          dword_5F8768[0] = 600;
          v7 = 0;
          goto LABEL_253;
        }
        if ( dword_92B0B4 != 4 )
          goto LABEL_253;
        if ( dword_4FEDF8 )
          free(dword_4FEDF8);
        v281 = qword_91CF30;
        v282 = dword_91CF28;
        v283 = dword_91CF2C;
        dword_4FEDF8 = 0;
        v428 = HIDWORD(qword_91CF30);
        sub_44A550(2034, qword_91CF30, HIDWORD(qword_91CF30));
        strcpy(MultiByteStr, v277);
        dword_4FEDAC = v428;
        dword_4FEDBC = v428;
        byte_4FEDD0 = 0;
        rcSrc2.left = word_6D98DC;
        rcSrc2.right = dword_4FEDE0 + word_6D98DC;
        rcSrc2.top = word_6D98DE;
        rcSrc2.bottom = dword_4FEDE4 + word_6D98DE;
        dword_4FEDA0 = v282;
        dword_4FEDA4 = v283;
        dword_4FEDA8 = v281;
        dword_4FEDB0 = v282;
        dword_4FEDB4 = v283;
        dword_4FEDB8 = v281;
        dword_4FED9C = 0;
        dword_4FED98 = 1;
        dword_4FEE1C = 0;
        memset(Str, 0, sizeof(Str));
        memset(v434, 0, sizeof(v434));
        strcpy(Str, v277);
        _strrev(Str);
        v284 = strstr(Str, SubStr);
        if ( v284 )
        {
          v285 = v284 + 1;
          v286 = (char *)(v434 - v285);
          do
          {
            v287 = *v285;
            v286[(_DWORD)v285] = *v285;
            ++v285;
          }
          while ( v287 );
          _strrev(v434);
        }
        else
        {
          v288 = (char *)(v434 - v277);
          do
          {
            v289 = *v277;
            v288[(_DWORD)v277] = *v277;
            ++v277;
          }
          while ( v289 );
        }
        v290 = &v433;
        while ( *++v290 )
          ;
        strcpy(v290, ".msk");
        v292 = sub_43B400(1802726702);
        sub_401310(v292, v434, &dword_4FEDF8, &dword_4FEDFC, 0, 0);
        goto LABEL_252;
      case 0x62:
        dword_4FB030 = 1;
        v7 = 0;
        instruction_ptr += 2;
        dword_4F5AB0 = 0;
        *(_DWORD *)&dword_92A488 = dword_92AE9C;
        dword_92A48C = dword_92AEA0;
        goto LABEL_253;
      case 0x63:
        v129 = *(_BYTE *)(v10 + 1) != 0;
        v130 = 84 * *(char *)v10;
        instruction_ptr += 4;
        dword_5F8894[v130] = v129;
        v7 = 0;
        goto LABEL_253;
      case 0x64:
        v393 = (double)*(__int16 *)(v10 + 1) * 0.01;
        v397 = 0.01 * (double)*(__int16 *)(v10 + 3);
        v384 = 0.1 * (double)*(__int16 *)(v10 + 5);
        sub_42D990(*(char *)v10, v393, v397, v384);
        if ( !dword_5F893C[84 * *(char *)v10] )
          goto LABEL_406;
        if ( v393 != 1.0 )
        {
          v131 = v393;
LABEL_410:
          dword_5F8930[84 * *(char *)v10] = 1;
          dword_5F8938[84 * *(char *)v10] = 1;
          dbl_5F8918[42 * *(char *)v10] = v131;
          dbl_5F8920[42 * *(char *)v10] = v397;
          v133 = *(char *)v10;
          instruction_ptr += 9;
          v7 = 0;
          dbl_5F8928[42 * v133] = v384;
          goto LABEL_253;
        }
        v131 = v393;
        if ( 1.0 != v397 || 0.0 != v384 )
          goto LABEL_410;
LABEL_406:
        dword_5F8930[84 * *(char *)v10] = 0;
        if ( dword_5F8960[84 * *(char *)v10] )
          free((void *)dword_5F8960[84 * *(char *)v10]);
        dword_5F8960[84 * *(char *)v10] = 0;
        sub_44A6E0();
        dbl_5F8920[42 * *(char *)v10] = 1.0;
        dbl_5F8918[42 * *(char *)v10] = 1.0;
        v132 = *(char *)v10;
        instruction_ptr += 9;
        v7 = 0;
        dbl_5F8928[42 * v132] = 0.0;
        goto LABEL_253;
      case 0x65:
        dword_4E6D6C = *(__int16 *)v10;
        dword_4E6D70 = *(__int16 *)(v10 + 2);
        if ( dword_5F888C[0] && dword_5F8930[0] )
        {
          sub_42D990(0, dbl_5F8918[0], dbl_5F8920[0], dbl_5F8928[0]);
          dword_5F8938[0] = 1;
        }
        if ( dword_5F89DC && dword_5F8A80 )
        {
          sub_42D990(1, dbl_5F8A68, dbl_5F8A70, dbl_5F8A78);
          dword_5F8A88 = 1;
        }
        if ( dword_5F8B2C && dword_5F8BD0 )
        {
          sub_42D990(2, dbl_5F8BB8, dbl_5F8BC0, dbl_5F8BC8);
          dword_5F8BD8 = 1;
        }
        if ( dword_5F8C7C && dword_5F8D20 )
        {
          sub_42D990(3, dbl_5F8D08, dbl_5F8D10, dbl_5F8D18);
          dword_5F8D28 = 1;
        }
        if ( dword_5F8DCC && dword_5F8E70 )
        {
          sub_42D990(4, dbl_5F8E58, dbl_5F8E60, dbl_5F8E68);
          dword_5F8E78 = 1;
        }
        if ( dword_5F8F1C && dword_5F8FC0 )
        {
          sub_42D990(5, dbl_5F8FA8, dbl_5F8FB0, dbl_5F8FB8);
          dword_5F8FC8 = 1;
        }
        instruction_ptr += 6;
        v7 = 0;
        goto LABEL_253;
      case 0x66:
        dword_5F8838[84 * *(char *)v10] = 1;
        dbl_5F8840[42 * *(char *)v10] = dbl_5F87C8[42 * *(char *)v10];
        dbl_5F8850[42 * *(char *)v10] = dbl_5F87D0[42 * *(char *)v10];
        dbl_5F8848[42 * *(char *)v10] = (double)*(__int16 *)(v10 + 1) * 0.01;
        dbl_5F8858[42 * *(char *)v10] = 0.01 * (double)*(__int16 *)(v10 + 3);
        *((double *)&unk_5F8878 + 42 * *(char *)v10) = dbl_5F87D8[42 * *(char *)v10];
        dword_5F8874[84 * *(char *)v10] = *(char *)(v10 + 5);
        *((double *)&unk_5F8880 + 42 * *(char *)v10) = 0.1 * (double)*(__int16 *)(v10 + 6);
        dword_5F8868[84 * *(char *)v10] = *(_DWORD *)(v10 + 8);
        dword_5F8860[84 * *(char *)v10] = dword_4E6D74;
        dword_5F8864[84 * *(char *)v10] = dword_4E6D78;
        dword_5F87E4[84 * *(char *)v10] = *(__int16 *)(v10 + 12);
        dword_5F886C[84 * *(char *)v10] = *(_DWORD *)(v10 + 14);
        dword_5F8870[84 * *(char *)v10] = *(_DWORD *)(v10 + 18);
        if ( !*(_BYTE *)v10 )
          dword_5F873C[0] = 1;
        if ( !dword_5F8868[84 * *(char *)v10] )
          dword_5F8838[84 * *(char *)v10] = 0;
        v7 = 0;
        instruction_ptr += 23;
        goto LABEL_253;
      case 0x67:
        v142 = *(_DWORD *)(v10 + 3) != 0;
        dword_5F9078 = *(_DWORD *)(v10 + 3);
        dword_5F9070 = 1;
        dword_5F907C = a1;
        dword_5F906C = v142;
        dword_5F9074 = *(_BYTE *)v10 != 0;
        dword_5F9068 = *(char *)(v10 + 1);
        if ( dword_4FB008 || dword_4FB00C || !dword_4FEC80 && *(_DWORD *)&dword_92A450 )
          sub_439EF0(2);
        instruction_ptr += 9;
        if ( dword_5F87E4[0] <= 0 )
        {
          if ( dword_91CA1C )
            sub_44A480(dword_5F87E4[0] != -1 ? 0xFFFFFF : 0);
        }
        else
        {
          v409 = (int)&unk_4F3ABC;
          v408 = (int)&unk_4F3ABC;
          v407 = dword_600524;
          v406 = 2015;
          v410 = 0;
          v411 = -2;
          sub_44AF20();
        }
        if ( !word_6D9910 )
          sub_42CE40(0);
        v143 = dword_5F8738;
        do
        {
          *v143 = 0;
          v143 += 84;
        }
        while ( (int)v143 < (int)&dword_5F9068 );
        goto LABEL_253;
      case 0x68:
        v392 = (double)*(__int16 *)v10 * 0.01;
        v396 = 0.01 * (double)*(__int16 *)(v10 + 2);
        if ( v392 >= 1.0 && (v78 = v392, v396 >= 1.0) )
        {
          v80 = v396;
        }
        else
        {
          v396 = 1.0;
          v392 = 1.0;
          v79 = (const CHAR *)sub_45F0F0(17);
          sub_4650C0(hWnd, 0, v79, aBg);
          v80 = 1.0;
          v78 = 1.0;
        }
        dword_4E6D74 = *(__int16 *)(v10 + 4);
        dword_4E6D78 = *(__int16 *)(v10 + 6);
        sub_42D7F0(v78, v80, dword_4E6D74, dword_4E6D78);
        if ( !dword_5F87EC || v392 == 1.0 || v396 == 1.0 )
        {
          dword_5F87E0[0] = 0;
          dbl_5F87D0[0] = 1.0;
          dword_4E6D74 = 400;
          dbl_5F87C8[0] = 1.0;
          dword_4E6D78 = 300;
        }
        else
        {
          dbl_5F87C8[0] = v392;
          dword_5F87E0[0] = 1;
          dbl_5F87D0[0] = v396;
          dword_5F87E8[0] = 1;
        }
        dbl_5F87D8[0] = 0.0;
        instruction_ptr += 10;
        dword_4E6D7C = dword_4E6D74;
        dword_4E6D80 = dword_4E6D78;
        v7 = 0;
        goto LABEL_253;
      case 0x69:
        v144 = *(char *)v10;
        instruction_ptr += 3;
        dword_92B0BC = v144;
        v7 = 0;
        goto LABEL_253;
      case 0x70:
        dword_4F50F8 = *(_DWORD *)(v10 + 3);
        v145 = *(_DWORD *)(v10 + 3);
        dword_4F50F0 = 1;
        dword_5F9070 = 1;
        dword_4F50EC = dword_4F50F8 != 0;
        dword_4F50FC = a1;
        dword_5F9078 = v145;
        dword_5F906C = v145 != 0;
        dword_4F50F4 = *(_BYTE *)v10 != 0;
        v146 = *(char *)(v10 + 1);
        dword_5F907C = a1;
        dword_4F50E8 = v146;
        dword_5F9074 = *(_BYTE *)v10 != 0;
        dword_5F9068 = *(char *)(v10 + 1);
        if ( dword_4FB008 || dword_4FB00C || !dword_4FEC80 && *(_DWORD *)&dword_92A450 )
          sub_439EF0(3);
        instruction_ptr += 9;
        if ( dword_5F8794[0] > 0 || dword_5F87E4[0] > 0 )
        {
          v409 = (int)&unk_4F3ABC;
          v408 = (int)&unk_4F3ABC;
          v407 = dword_600524;
          v406 = 2015;
          v410 = 0;
          v411 = -2;
          sub_44AF20();
        }
        else
        {
          if ( dword_5F8794[0] == -1 || dword_5F87E4[0] == -1 )
            v147 = 0;
          else
            v147 = 0xFFFFFF;
          if ( dword_91CA1C )
            sub_44A480(v147);
        }
        if ( !word_6D9910 )
          sub_42CE40(0);
        v148 = dword_5F8738;
        do
        {
          *v148 = 0;
          v148 += 84;
        }
        while ( (int)v148 < (int)&dword_5F9068 );
        goto LABEL_253;
      case 0x71:
        if ( !sub_42CAE0(v10) )
        {
          if ( !dword_92B1FC )
          {
            v380 = (char *)v10;
            goto LABEL_1073;
          }
          goto LABEL_1074;
        }
LABEL_196:
        instruction_ptr += strlen((const char *)v10) + 2;
LABEL_197:
        v7 = 0;
        goto LABEL_253;
      case 0x72:
        sub_42CC40();
        instruction_ptr += 2;
        v7 = 0;
        goto LABEL_253;
      case 0x73:
        v82 = (char *)(v10 + 9);
        do
        {
          v83 = *v82;
          String2[(_DWORD)v82 - 9 - v10] = *v82;
          ++v82;
        }
        while ( v83 );
        v84 = (char *)&v423 + 3;
        while ( *++v84 )
          ;
        strcpy(v84, ".wip");
        v86 = sub_43B400(&dword_4FEDE0);
        sub_467D80(v86, Xa, v404, 1, 0);
        dword_4FEDA0 = 0;
        dword_4FEDA4 = 0;
        dword_4FEDA8 = dword_4FEDE0;
        dword_4FEDAC = dword_4FEDE4;
        dword_4FEDB0 = 0;
        dword_4FEDB4 = 0;
        dword_4FEDB8 = dword_4FEDE0;
        dword_4FEDBC = dword_4FEDE4;
        v87 = *(unsigned __int16 *)v10;
        if ( (*(_BYTE *)(v10 + 8) & 1) != 0 )
          LOWORD(v87) = heap[v87];
        v88 = (__int16)v87;
        rcSrc2.left = (__int16)v87;
        v89 = *(unsigned __int16 *)(v10 + 2);
        if ( (*(_BYTE *)(v10 + 8) & 2) != 0 )
          LOWORD(v89) = heap[v89];
        rcSrc2.top = (__int16)v89;
        rcSrc2.right = dword_4FEDE0 + v88;
        rcSrc2.bottom = dword_4FEDE4 + (__int16)v89;
        if ( !_stricmp((const char *)(v10 + 9), &byte_4FEDD0) && dword_4FED9C == *(_DWORD *)(v10 + 4) )
          goto LABEL_323;
        if ( dword_4FEDF8 )
          free(dword_4FEDF8);
        dword_4FEDF8 = 0;
        dword_4FED9C = *(_DWORD *)(v10 + 4);
        v90 = (char *)(v10 + 9);
        do
        {
          v91 = *v90;
          *(&byte_4FEDD0 + (_DWORD)v90 - v10 - 9) = *v90;
          ++v90;
        }
        while ( v91 );
        v92 = (const char *)sub_43B400(String2);
        if ( !sub_4011D0(2034, dword_4FED9C, v92, v370, 0) )
        {
LABEL_805:
          if ( dword_92B1FC )
            goto LABEL_1074;
LABEL_1084:
          v358 = (const char *)sub_45F0F0(15);
          sprintf(v432, v358, String2);
          v376 = (const CHAR *)sub_45F0F0(17);
          sub_4650C0(hWnd, 0x10u, v376, v432);
          goto LABEL_1074;
        }
        v94 = 0;
        do
        {
          v95 = *(&byte_4FEDD0 + v94);
          String2[v94++] = v95;
        }
        while ( v95 );
        v96 = (char *)&v423 + 3;
        while ( *++v96 )
          ;
        LOBYTE(v93) = 0;
        strcpy(v96, ".msk");
        v98 = sub_43B400(v93);
        sub_401310(v98, String2, &dword_4FEDF8, &dword_4FEDFC, 0, 0);
LABEL_323:
        dword_4FED98 = 1;
        dword_4FEE1C = 0;
        instruction_ptr += strlen((const char *)(v10 + 9)) + 11;
        v7 = 0;
        goto LABEL_253;
      case 0x74:
        dword_4FED98 = *(_BYTE *)v10 != 0;
        if ( dword_92B0B4 == 4 )
        {
          dword_92B0B4 = 0;
          sub_459350();
          if ( dword_4FEDF8 )
            free(dword_4FEDF8);
          dword_4FEDF8 = 0;
          sub_44A6E0();
        }
        instruction_ptr += 3;
        v7 = 0;
        goto LABEL_253;
      case 0x75:
        dword_4FEDB0 = *(__int16 *)v10;
        dword_4FEDB4 = *(__int16 *)(v10 + 2);
        dword_4FEDB8 = dword_4FEDB0 + *(__int16 *)(v10 + 4);
        v99 = dword_4FEDB4 + *(__int16 *)(v10 + 6);
        v100 = rcSrc2.top + *(__int16 *)(v10 + 6);
        dword_4FEDBC = v99;
        rcSrc2.right = rcSrc2.left + dword_4FEDB8 - dword_4FEDB0;
        rcSrc2.bottom = v100;
        if ( dword_4FEDB8 > dword_4FEDA8 || v99 > dword_4FEDAC )
        {
          v101 = (const CHAR *)sub_45F0F0(17);
          sub_4650C0(hWnd, 0, v101, aS, byte_4F88E0);
          dword_4FED98 = 0;
        }
        instruction_ptr += 10;
        v7 = 0;
        goto LABEL_253;
      case 0x76:
        dword_4FEE28 = *(__int16 *)v10;
        dword_4FEE2C = *(__int16 *)(v10 + 2);
        dword_4FEE30 = *(_DWORD *)(v10 + 4);
        dword_4FEE38 = *(char *)(v10 + 8);
        dword_4FEE3C = *(__int16 *)(v10 + 10);
        dword_4FEE40 = *(_DWORD *)(v10 + 12);
        dword_4FEE20 = rcSrc2.left;
        dword_4FEE24 = rcSrc2.top;
        dword_4FEE34 = timeGetTime();
        if ( dword_4FEE30 )
          dword_4FEE1C = 1;
        if ( dword_4FB008 || dword_4FB00C || !dword_4FEC80 && *(_DWORD *)&dword_92A450 )
        {
          v102 = *(_BYTE *)(v10 + 9);
          if ( v102 || dword_4FEC80 || *(_DWORD *)&dword_92A450 != 1 )
          {
            if ( v102 == 2 )
              dword_4FEE30 = 0;
          }
          else
          {
            dword_4FEE30 /= 3u;
          }
        }
        instruction_ptr += 18;
        goto LABEL_346;
      case 0x77:
        dword_4FEE0C = *(__int16 *)v10 - dword_4FEDB0;
        dword_4FEE10 = *(__int16 *)(v10 + 2) - dword_4FEDB4;
        dword_4FEE14 = *(_DWORD *)(v10 + 4);
        dword_4FEE04 = dword_4FEDB0;
        dword_4FEE08 = dword_4FEDB4;
        dword_4FEE18 = timeGetTime();
        if ( dword_4FEE14 )
          dword_4FEE00 = 1;
        instruction_ptr += 10;
        sub_42CE40(0);
        goto LABEL_253;
      case 0x78:
        if ( dword_4FEC80 || !*(_DWORD *)&dword_92A450 )
          dword_4F6E90 = 1;
        dword_4F6EA0 = 0;
        dword_4F6E9C = 0;
        dword_4F6EA4 = *(unsigned __int8 *)v10;
        LOBYTE(dword_4F6E98) = *(_BYTE *)(v10 + 1);
        dword_4F6E94 = *(_BYTE *)(v10 + 2) != 0;
        v308 = *(_DWORD *)(v10 + 3);
        instruction_ptr += 9;
        dword_4F6EA8 = v308;
        dword_92B2F0 = 0;
        dword_92B2E0 = 0;
        dword_92B2E4 = 100 * dword_4F6EA4;
        dword_92B2EC = 0;
        dword_92B2D8 = -1;
        dword_92B2E8 = 1;
        if ( !(100 * dword_4F6EA4) )
          goto LABEL_197;
        dword_92B2DC = 1;
        v7 = 0;
        goto LABEL_253;
      case 0x79:
        dword_4F6E90 = 0;
        sub_418980(1);
        instruction_ptr += 2;
        dword_4FB030 = 1;
        dword_6DB404 = 0;
        dword_92B2DC = 0;
        dword_92B2E0 = 0;
        dword_92B2E4 = 0;
        v7 = 0;
        goto LABEL_253;
      case 0x81:
        instruction_ptr += 3;
        v7 = 0;
        goto LABEL_253;
      case 0x82:
        if ( dword_4FB008 || dword_4FB00C )
        {
          v7 = 0;
        }
        else
        {
          dword_92B140 = 1;
          dword_4F50E4 = timeGetTime();
          dword_4F50DC = 1;
          dword_92B148 = *(__int16 *)v10;
          if ( dword_92B148 )
            dword_92B144 = timeGetTime();
        }
        if ( !word_6D9910 )
          sub_42CE40(0);
        instruction_ptr += 4;
        goto LABEL_253;
      case 0x83:
        if ( !word_6D9912 )
          dword_4FEC3C = 1;
        sub_422A70(1, v405 == 1);
        instruction_ptr += 2;
        goto LABEL_253;
      case 0x84:
        if ( !word_6D9912 )
          dword_4FEC3C = 1;
        sub_42CE40(0);
        dword_4FAFE0 = 0;
        dword_5004E8 = 0;
        sub_422A70(0, v405 == 1);
        instruction_ptr += 2;
        goto LABEL_253;
      case 0x85:
        v293 = *(char *)v10;
        instruction_ptr += 3;
        dword_4FEC30 = v293;
        v7 = 0;
        goto LABEL_253;
      case 0x86:
        word_6D988E = *(_WORD *)&dword_92AE98;
        word_6D9914 = dword_4FB008 != 0 || dword_92B168 != 0;
        if ( *((_DWORD *)dword_804634 + 3) )
          v294 = sub_463920();
        else
          v294 = 0;
        instruction_ptr += 3;
        v7 = 0;
        word_6D9890 = v294 != 0;
        word_6D9892 = dword_4F88D0 != 0;
        goto LABEL_253;
      case 0x87:
        v299 = dword_4FB00C || dword_4FB008;
        v300 = *(__int16 *)v10;
        instruction_ptr += 4;
        heap[v300] = v299;
        v7 = 0;
        goto LABEL_253;
      case 0x88:
        instruction_ptr += 4;
        v7 = 0;
        word_6D98C0 = dword_4F50F0 != 0;
        word_6D98C2 = dword_5F9070 != 0;
        goto LABEL_253;
      case 0x89:
        dword_4FB00C = 0;
        dword_4FB008 = 0;
        memset(&dword_6DB528, 0, 0x12900Cu);
        instruction_ptr += 2;
        dword_4F50D8 = 0;
        dword_4F50DC = 0;
        dword_4F50E0 = 0;
        dword_4F50E4 = 0;
        v7 = 0;
        goto LABEL_253;
      case 0x8A:
        sub_432E10(1);
        instruction_ptr += 2;
        goto LABEL_253;
      case 0x8B:
        sub_4258F0(1, 1);
        instruction_ptr += 2;
        goto LABEL_253;
      case 0x8C:
        v298 = *(unsigned __int16 *)v10;
        instruction_ptr += 4;
        dword_4FB000 = v298;
        v7 = 0;
        goto LABEL_253;
      case 0x8D:
        v136 = v405;
        dword_4FEC50 = 1;
        if ( sub_422060(2 * (v405 != 1) - 1, 3) && v136 == 1 )
          sub_45EEB0(250);
        instruction_ptr += 2;
        goto LABEL_253;
      case 0x8E:
        if ( !word_6D990E )
          dword_92B110 = 1;
        instruction_ptr += 2;
        v7 = 0;
        goto LABEL_253;
      case 0xA0:
        v76 = *(unsigned __int16 *)v10;
        if ( (*(_BYTE *)(v10 + 4) & 1) != 0 )
          v76 = heap[v76];
        dword_5F874C[0] = v76;
        if ( (*(_BYTE *)(v10 + 4) & 2) != 0 )
          v77 = heap[*(unsigned __int16 *)(v10 + 2)];
        else
          v77 = *(unsigned __int16 *)(v10 + 2);
        instruction_ptr += 7;
        dword_5F8750[0] = v77;
        dword_5F8754[0] = v76 + 800;
        dword_5F8758[0] = v77 + 600;
        if ( dword_5F87E0[0] )
          dword_5F87E8[0] = 1;
        v7 = 0;
        goto LABEL_253;
      case 0xA1:
        sub_4529B0(&v420, &v419);
        v126 = *(unsigned __int16 *)(v10 + 1);
        if ( (*(_BYTE *)(v10 + 5) & 1) != 0 )
          LOWORD(v126) = heap[v126];
        dword_5F88AC[84 * *(char *)v10] = (__int16)v126;
        v127 = *(unsigned __int16 *)(v10 + 3);
        if ( (*(_BYTE *)(v10 + 5) & 2) != 0 )
          LOWORD(v127) = heap[v127];
        dword_5F88B0[84 * *(char *)v10] = (__int16)v127;
        dword_5F88B4[84 * *(char *)v10] = v420 + dword_5F88AC[84 * *(char *)v10];
        v128 = *(char *)v10;
        instruction_ptr += 8;
        dword_5F88B8[84 * v128] = v419 + dword_5F88B0[84 * v128];
        if ( dword_5F8930[84 * *(char *)v10] )
          dword_5F8938[84 * *(char *)v10] = 1;
        v7 = 0;
        goto LABEL_253;
      case 0xA2:
        v345 = *(__int16 *)(v10 + 1);
        v346 = 13 * *(char *)v10;
        v347 = *(__int16 *)(v10 + 3);
        instruction_ptr += 7;
        v348 = &dword_92B328[v346];
        *v348 = 1;
        v348[2] = v345;
        v348[3] = v347;
        v7 = 0;
        goto LABEL_253;
      case 0xA3:
        v349 = *(__int16 *)(v10 + 3);
        v416 = *(__int16 *)(v10 + 1);
        v417 = v349;
        sub_465860(&v416);
        instruction_ptr += 7;
        v7 = 0;
        goto LABEL_253;
      case 0xA4:
        v350 = *(__int16 *)(v10 + 3);
        v416 = *(__int16 *)(v10 + 1);
        v417 = v350;
        sub_4658A0(&v416);
        instruction_ptr += 7;
        v7 = 0;
        goto LABEL_253;
      case 0xA5:
        v351 = &dword_92B328[13 * *(char *)v10];
        *v351 = 0;
        v351[1] = 0;
        v351[5] = 0;
        v351[6] = 0;
        v351[7] = 0;
        sub_418980(1);
        instruction_ptr += 3;
        dword_4FB030 = 1;
        v7 = 0;
        goto LABEL_253;
      case 0xA6:
        if ( dword_92B0B4 == 3 )
        {
          dword_92B0B4 = 0;
          sub_459350();
          dword_5F873C[0] = 0;
        }
        if ( dword_92B0B4 == 4 )
        {
          dword_92B0B4 = 0;
          sub_459350();
          if ( dword_4FEDF8 )
            free(dword_4FEDF8);
          dword_4FEDF8 = 0;
          dword_4FED98 = 0;
        }
        v7 = 0;
        instruction_ptr += 2;
        goto LABEL_253;
      case 0xA7:
        sub_466720(&dword_92B588, 16, aCross);
        instruction_ptr += 2;
        v7 = 0;
        goto LABEL_253;
      case 0xA8:
        if ( dword_92B0F8 )
          sub_45AC40(
            *(_BYTE *)v10,
            *(_BYTE *)(v10 + 1),
            *(_BYTE *)(v10 + 2),
            *(_WORD *)(v10 + 7),
            *(_WORD *)(v10 + 9),
            *(_WORD *)(v10 + 11),
            *(_WORD *)(v10 + 13));
        instruction_ptr += 17;
        v7 = 0;
        goto LABEL_253;
      case 0xA9:
        if ( dword_92B0F8 )
          sub_45AD00();
        instruction_ptr += 2;
        v7 = 0;
        goto LABEL_253;
      case 0xAA:
        memset(::DestStr, 0, sizeof(::DestStr));
        memset(DestStr, 0, sizeof(DestStr));
        v352 = *(char *)v10;
        if ( *(char *)v10 < 12 )
        {
          v353 = (const char *)&unk_4C8240;
        }
        else
        {
          v352 -= 12;
          v353 = (const char *)&unk_4C8244;
        }
        sprintf(SrcStr, "%s%02d", v353, v352);
        LCMapStringA(0x800u, (DWORD)&unk_800000, SrcStr, strlen(SrcStr), ::DestStr, 64);
        sprintf(::DestStr, "%s:", ::DestStr);
        sprintf(SrcStr, "%02d", *(char *)(v10 + 1));
        memset(DestStr, 0, sizeof(DestStr));
        LCMapStringA(0x800u, (DWORD)&unk_800000, SrcStr, strlen(SrcStr), DestStr, 64);
        sprintf(::DestStr, "%s%s", ::DestStr, DestStr);
        sub_466720(&dword_92B250, 48, ::DestStr);
        instruction_ptr += 4;
        v7 = 0;
        goto LABEL_253;
      case 0xAB:
        instruction_ptr += 2;
        word_6D9198 = dword_4F6ED8;
        word_6D919A = dword_4F6ED4;
        v7 = 0;
        goto LABEL_253;
      case 0xAC:
        if ( dword_4FB008 || dword_4FB00C )
        {
          sub_466C70();
          sub_466C70();
          instruction_ptr += 2;
          v7 = 0;
        }
        else
        {
          sub_466D60();
          sub_466D60();
          instruction_ptr += 2;
          v7 = 0;
        }
        goto LABEL_253;
      case 0xAD:
        v408 = word_6D98CC;
        v411 = word_6D98D2;
        v406 = word_6D98C8;
        v407 = word_6D98CA;
        v409 = word_6D98CE;
        v410 = word_6D98D0;
        v355 = *(_DWORD *)(v10 + 1);
        v412 = word_6D98D4;
        v356 = *(char *)v10;
        v413 = word_6D98D6;
        sub_459D80(v356, v355, *(_DWORD *)(v10 + 5), &v406, word_6D98D8 != 0);
        v7 = 0;
        instruction_ptr += 11;
        goto LABEL_253;
      case 0xAE:
        sub_459F80();
        instruction_ptr += 2;
        v7 = 0;
        goto LABEL_253;
      case 0xB1:
        dword_4E6D7C = *(__int16 *)v10;
        v149 = *(__int16 *)(v10 + 2);
        instruction_ptr += 6;
        dword_4E6D80 = v149;
        v7 = 0;
        goto LABEL_253;
      case 0xB2:
        if ( dword_92B0C4 && !sub_45C350(*(char *)v10, v10 + 2) )
        {
          sub_45C420();
          if ( dword_92B1FC )
            goto LABEL_1074;
          v363 = v10 + 2;
LABEL_1095:
          v364 = (const char *)sub_45F0F0(15);
          sprintf(v432, v364, v363);
          v377 = (const CHAR *)sub_45F0F0(17);
          sub_4650C0(hWnd, 0x10u, v377, v432);
          goto LABEL_1074;
        }
        dword_4FB030 = 1;
        instruction_ptr += strlen((const char *)(v10 + 2)) + 4;
        v7 = 0;
        goto LABEL_253;
      case 0xB3:
        if ( dword_92B0C4 )
          sub_45C420();
        instruction_ptr += 3;
        v7 = 0;
        goto LABEL_253;
      case 0xB4:
        if ( (!dword_4FB008 && !dword_4FB00C || *(_BYTE *)(v10 + 10)) && dword_92B0C4 )
        {
          v310 = (dword_4FB008 || dword_4FB00C) && *(_BYTE *)(v10 + 10) == 2;
          sub_45C6A0(*(__int16 *)(v10 + 2), *(__int16 *)(v10 + 4), *(_DWORD *)(v10 + 6), v310);
        }
        instruction_ptr += 13;
        v7 = 0;
        goto LABEL_253;
      case 0xB5:
        if ( !dword_92B0C4 )
          goto LABEL_968;
        if ( dword_4FB008 || dword_4FB00C )
        {
          v311 = *(char *)v10;
          if ( v311 <= 0xF )
          {
            v6 = *((_DWORD *)dword_92B0C4 + 18 * v311 + 72) == 0;
            v312 = (char *)dword_92B0C4 + 72 * v311 + 272;
            if ( !v6 )
            {
              *((_DWORD *)v312 + 4) = 0;
              *((_DWORD *)v312 + 3) = 1;
            }
          }
LABEL_968:
          instruction_ptr += 8;
          v7 = 0;
        }
        else
        {
          sub_45C6E0(*(char *)(v10 + 1));
          instruction_ptr += 8;
          v7 = 0;
        }
        goto LABEL_253;
      case 0xB6:
        if ( !dword_80461C || !dword_4FAFE0 )
        {
          if ( Count )
          {
            if ( dword_804618 )
            {
              if ( dword_804628 )
                --dword_804628;
              if ( dword_804624 )
                --dword_804624;
            }
            v391 = *(_WORD *)v10;
            v28 = (char *)(v10 + 2);
            do
            {
              v29 = *v28;
              v436[(_DWORD)v28 - 2 - v10] = *v28;
              ++v28;
            }
            while ( v29 );
            v383 = strlen(&Source);
            strcat(&Source, v436);
            dword_4FB044 = 0;
            dword_4FB048 = 0;
            dword_4FB040 = 0;
            dword_4FB03C = 0;
            dword_4FB034 = 0;
            dword_4FB038 = 1;
            dword_4FB080 = 0;
            dword_4FB02C = 0;
            dword_4F50DC = 0;
            dword_4F50E0 = strlen(v436);
            dword_4FB01C = 1;
            dword_4FB014 = 1;
            dword_80462C = 0;
            instruction_ptr += strlen((const char *)(v10 + 2)) + 4;
            sub_42AC00(1);
            do_something_perhaps_validation_with_string(&Source, 1, v383);
            sub_42BFA0(&Source);
            sub_43CB20();
            if ( dword_4FB064 == 3 )
            {
              if ( !dword_4FB038 )
                sub_421780(&dword_4FB094);
            }
            else if ( !dword_4FB038 )
            {
              v30 = byte_92B614 == 35 || byte_92B614 == 39;
              sub_421080(v30, 0, 0);
            }
            dword_4FB024 = sub_42C8B0(v391, 1);
            if ( !*(_DWORD *)&dword_92A3EC || !sub_42C8B0(v391, 0) || (dword_4FB00C = 1, dword_4FB010) )
              dword_4FB00C = 0;
            if ( *(_DWORD *)&dword_92A3D4 )
            {
              if ( dword_4FB008 )
              {
                if ( !a2 && !sub_42C8B0(v391, 1) )
                {
                  dword_4FB008 = 0;
                  if ( byte_92B614 == 35 || byte_92B614 == 39 )
                    sub_424B70(0);
                }
              }
            }
            dword_4FB010 = 0;
            sub_42C820(0);
            sub_418980(0);
            dword_4FB030 = 1;
            if ( !dword_4FAFD8 )
              sub_42CDD0();
          }
          else
          {
            v26 = (const char *)(v10 + 2);
            instruction_ptr += strlen(v26) + 4;
            v27 = (const CHAR *)sub_45F0F0(17);
            sub_4650C0(hWnd, 0, v27, &byte_4C80B0, v26);
          }
        }
        goto LABEL_253;
      case 0xB7:
        v157 = _stricmp((const char *)(v10 + 5), &byte_6DB314[44 * *(char *)v10]);
        dword_6DB304[11 * *(char *)v10] = *(__int16 *)(v10 + 1);
        dword_6DB308[11 * *(char *)v10] = *(__int16 *)(v10 + 3);
        if ( !v157 )
          goto LABEL_542;
        if ( dword_6DB30C[11 * *(char *)v10] )
          free((void *)dword_6DB30C[11 * *(char *)v10]);
        dword_6DB30C[11 * *(char *)v10] = 0;
        v158 = (char *)(v10 + 5);
        v159 = &String2[-v10 - 5];
        do
        {
          v160 = *v158;
          v158[(_DWORD)v159] = *v158;
          ++v158;
        }
        while ( v160 );
        v161 = (char *)&v423 + 3;
        while ( *++v161 )
          ;
        strcpy(v161, ".wip");
        v163 = sub_43B400(v381);
        if ( !sub_4014F0(*(char *)v10 + 2021, 1, &v414, v404, v163) )
          goto LABEL_1083;
        dword_6DB2FC[11 * *(char *)v10] = v414;
        dword_6DB300[11 * *(char *)v10] = v415;
        v164 = (char *)(v10 + 5);
        do
        {
          v165 = *v164;
          v164[(_DWORD)v159] = *v164;
          ++v164;
        }
        while ( v165 );
        v166 = (char *)&v423 + 3;
        while ( *++v166 )
          ;
        strcpy(v166, ".msk");
        v168 = 44 * *(char *)v10;
        X_4 = (char *)&unk_6DB310 + v168;
        Xb = &dword_6DB30C[v168 / 4];
        v169 = sub_43B400((char *)&unk_6DB310 + v168);
        if ( !sub_401310(v169, String2, Xb, X_4, 0, 0) )
          goto LABEL_1085;
        v170 = (char *)(v10 + 5);
        v171 = &byte_6DB314[44 * *(char *)v10];
        do
        {
          v172 = *v170;
          *v171++ = *v170++;
        }
        while ( v172 );
LABEL_542:
        dword_6DB2F8[11 * *(char *)v10] = 1;
        v7 = 0;
        instruction_ptr += strlen((const char *)(v10 + 5)) + 7;
        goto LABEL_253;
      case 0xB8:
        v173 = *(_BYTE *)(v10 + 1) != 0;
        v174 = 11 * *(char *)v10;
        v7 = 0;
        instruction_ptr += 4;
        dword_6DB2F8[v174] = v173;
        goto LABEL_253;
      case 0xB9:
        v103 = *(char *)(v10 + 1);
        v104 = *(char *)v10;
        instruction_ptr += 4;
        dword_92B170[v104] = v103;
        v7 = 0;
        goto LABEL_253;
      case 0xBA:
        v315 = *(__int16 *)v10;
        v316 = dword_92B0C8;
        *((_DWORD *)dword_92B0C8 + 4) = *(__int16 *)(v10 + 2);
        v316[3] = v315;
        if ( !sub_45F3A0(
                *(unsigned __int8 *)(v10 + 4),
                *(unsigned __int8 *)(v10 + 5),
                *(unsigned __int8 *)(v10 + 6),
                dword_4FB050,
                *(unsigned __int8 *)(v10 + 7),
                *(unsigned __int8 *)(v10 + 8),
                *(__int16 *)(v10 + 9))
          && !dword_92B1FC )
        {
          v317 = (const char *)sub_45F0F0(15);
          sprintf(v432, v317, v10 + 11);
          v318 = (const CHAR *)sub_45F0F0(17);
          sub_4650C0(hWnd, 0x10u, v318, v432);
        }
        v62 = instruction_ptr + strlen((const char *)(v10 + 11)) + 13;
        goto LABEL_251;
      case 0xBB:
        sub_45F4D0();
        instruction_ptr += 2;
        v7 = 0;
        goto LABEL_253;
      case 0xBC:
        v319 = *(char *)v10;
        v320 = *(char *)(v10 + 2);
        v321 = *(char *)(v10 + 1);
        v322 = 84 * v319;
        v395 = v320;
        v389 = v321;
        if ( dword_5F8790[84 * v319] && dword_5F878C[84 * *(char *)v10] != v321 )
          goto LABEL_1002;
        v323 = funcs_40FAF2[v321];
        if ( *(_BYTE *)v10 )
        {
          if ( !v323((int)off_4F3AD0[v319], dword_5F8770[84 * *(char *)v10], v320) )
            goto LABEL_1002;
          if ( dword_5F8740[84 * v319] )
          {
            sprintf(v425, "%s.wip", (const char *)&unk_600538 + 126832 * v319);
            sub_43B620(v429);
            v326 = 1;
            v401 = sub_467CD0();
            if ( v401 > 1 )
            {
              while ( funcs_40FAF2[v389]((int)off_4F3AD0[v319] + v326, 0, v395) )
              {
                if ( ++v326 >= v401 )
                  goto LABEL_1000;
              }
LABEL_1002:
              v379 = *(char *)v10;
              v327 = (const CHAR *)sub_45F0F0(17);
              sub_4650C0(hWnd, 0, v327, aD_0, v379);
              goto LABEL_1003;
            }
          }
        }
        else
        {
          if ( !v323(14, 0, v320) )
            goto LABEL_1002;
          if ( dword_5F8740[0] )
          {
            sprintf(v425, "%s.wip", (const char *)&unk_600538);
            ((void (*)(void))sub_43B400)();
            v324 = sub_467CD0();
            v325 = 1;
            if ( v324 > 1 )
            {
              while ( funcs_40FAF2[v389](v325 + 14, 0, v395) )
              {
                if ( ++v325 >= v324 )
                  goto LABEL_1000;
              }
              goto LABEL_1002;
            }
          }
        }
LABEL_1000:
        dword_5F8790[v322] += v395;
        v6 = dword_5F87E0[v322] == 0;
        dword_5F878C[v322] = v389;
        if ( !v6 )
        {
          instruction_ptr += 5;
          dword_5F87E8[v322] = 1;
          goto LABEL_252;
        }
LABEL_1003:
        instruction_ptr += 5;
        goto LABEL_252;
      case 0xBD:
        v313 = *(_BYTE *)v10 != 0;
        instruction_ptr += 3;
        v7 = 0;
        dword_92B158 = v313;
        goto LABEL_253;
      case 0xBE:
        sub_436BB0(*(char *)v10, *(char *)(v10 + 1));
        instruction_ptr += 4;
        v7 = 0;
        goto LABEL_253;
      case 0xBF:
        if ( dword_4F6E90 )
        {
          dword_4F6EA4 = *(unsigned __int8 *)v10;
          LOBYTE(dword_4F6E98) = *(_BYTE *)(v10 + 1);
          dword_4F6E94 = *(_BYTE *)(v10 + 2) != 0;
          dword_4F6EA8 = *(_DWORD *)(v10 + 3);
          dword_92B2E4 = 100 * dword_4F6EA4;
        }
        instruction_ptr += 9;
        v7 = 0;
        goto LABEL_253;
      case 0xE0:
        sub_466C70();
        memset(aCross, 0, 0x4Fu);
        if ( strlen((const char *)v10) <= 0x4E )
        {
          strcpy(aCross, (const char *)v10);
          sub_42AC00(0);
        }
        else
        {
          v307 = (const CHAR *)sub_45F0F0(17);
          sub_4650C0(hWnd, 0x10u, v307, &byte_4C818C);
        }
        instruction_ptr += strlen((const char *)v10) + 2;
        goto LABEL_252;
      case 0xE2:
        instruction_ptr += 2;
        v372 = (unsigned int)v15;
        v295 = sub_437140(v15);
        sub_419A90(v295, HIDWORD(v372));
        goto LABEL_253;
      case 0xE3:
        if ( !dword_4FEC30 )
        {
          sub_418980(1);
          sub_4348B0(1);
          if ( sub_452630(&dword_4FEC28) )
            dword_4FEC48 = 1;
          sub_4348B0(1);
          sub_418980(1);
          v373 = v296;
          v297 = sub_437110(v296);
          sub_419410(v297, HIDWORD(v373));
          dword_4FEC48 = 0;
          if ( dword_4FEC28 )
          {
            free(dword_4FEC28);
            dword_4FEC28 = 0;
          }
        }
        v7 = 0;
        instruction_ptr += 2;
        goto LABEL_253;
      case 0xE4:
        dword_804614 = *(char *)v10;
        v6 = *(_BYTE *)v10 == 0;
        dword_804630 = 0;
        instruction_ptr += 3;
        dword_804624 = 0;
        dword_804628 = 0;
        dword_80462C = 0;
        dword_804618 = !v6;
        if ( !v6 )
        {
          if ( dword_91AAB8 )
            sub_44A480(0);
          memset(dword_4F5104, 0, 0x75300u);
        }
        if ( byte_92B614 == 74 )
        {
          dword_80461C = 0;
          v7 = 0;
        }
        else
        {
          dword_4FB014 = 0;
          dword_80461C = 1;
          dword_4FB030 = 1;
          sub_42CE40(0);
          v309 = *(_BYTE *)instruction_ptr;
          if ( *(_BYTE *)instruction_ptr != 65 && v309 != 66 && v309 != 74 )
            v7 = 0;
        }
        goto LABEL_253;
      case 0xE5:
        instruction_ptr += 2;
        dword_4FB030 = 1;
        dword_804630 = 0;
        dword_804624 = 0;
        dword_804628 = 0;
        v7 = 0;
        goto LABEL_253;
      case 0xE6:
        instruction_ptr += 3;
        goto LABEL_253;
      case 0xE7:
        v354 = *(__int16 *)v10;
        if ( !byte_804638[v354] )
          byte_804638[v354] = 1;
        instruction_ptr += 4;
        v7 = 0;
        goto LABEL_253;
      case 0xE8:
        sub_418980(1);
        dword_4FB030 = 1;
        instruction_ptr += strlen((const char *)v10) + 2;
        v7 = 0;
        goto LABEL_253;
      case 0xE9:
        sub_418980(1);
        instruction_ptr += 2;
        dword_4FB030 = 1;
        v7 = 0;
        goto LABEL_253;
      case 0xEA:
        if ( sub_43DA50(*(char *)v10) )
        {
          instruction_ptr += strlen((const char *)(v10 + 1)) + 3;
LABEL_252:
          v7 = 0;
          goto LABEL_253;
        }
        if ( dword_92B1FC )
          goto LABEL_1074;
        v363 = v10 + 1;
        goto LABEL_1095;
      case 0xEB:
        sub_43DA00();
        instruction_ptr += 2;
        v7 = 0;
        goto LABEL_253;
      case 0xFF:
        v374 = aS_1;
        goto LABEL_971;
      default:
        v374 = (const CHAR *)&unk_4C8264;
LABEL_971:
        v314 = (const CHAR *)sub_45F0F0(17);
        sub_4650C0(hWnd, 0, v314, v374, byte_4F88E0);
        if ( !dword_600518 )
          goto LABEL_972;
        goto LABEL_253;
    }
  }
}