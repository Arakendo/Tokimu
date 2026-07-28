//******************************************************
//                   SortDetailInfo
//
// Passed:
//    the array to sort
// Process:
//    Uses a bubble sort to sort the array in test case name order
//******************************************************
function SortDetailInfo(DetailInfo){
   tmp = new DetailRecord();
   Size = DetailInfo.length;
   for (var i=0;i<Size;i++){
      for (var j = 0; j<i;j++){
	   if (DetailInfo[i].TestName < DetailInfo[j].TestName){
		tmp = DetailInfo[i];
		DetailInfo[i] = DetailInfo[j];
		DetailInfo[j] = tmp;
	   }
      }
   }
}

//******************************************************
//            BuildDetailInfo
//
//  Passed:
//	Node 		to look at
//	DetailInfo 	Array to hold all data
//	count		indicator of tree level
//	addnode		node data to insert into DetailInfo
//  Process:
//	RECURSION (very tricky)
//  Note:
//	HOLD had to be created at each level to restore 
//		list to previous state.
//******************************************************
function BuildDetailInfo(Node,count,addnode){
   count = count + 1;
   list = Node.childNodes;
   var HOLD = list;
   for (var i=0;i<list.length;i++){
      name = list(i).getAttribute ("Name");
      value = list(i).text;
      check = count+"-"+list(i).nodeName;
      switch (check){
	case "1-Version":{
	   addnode.Version = name;
	   BuildDetailInfo(list.item(i),count,addnode);
	   break;
        }
	case "2-CGMCategory":{
	   addnode.CGMCategory = name;
	   BuildDetailInfo(list.item(i),count,addnode);
	   addnode.CatNote = ""; //leaving Category so clear CatNot
	   addnode.CGMElement = "";
	   break;
	}
	case "3-CGMElement":{
	   if (addnode.CGMElement == "")
	   	addnode.CGMElement = name;
	   else
		addnode.CGMElement = addnode.CGMElement + "<P>" + name;
	   break;
	}
        case "3-CatNote":{
	   if (addnode.CatNote == "")
	   	addnode.CatNote = name;
	   else
		addnode.CatNote = addnode.CatNote+ "<P>"+name;
	   break;
	}
	case "3-TestCase":{
	   addnode.TestName = name;
	   BuildDetailInfo(list.item(i),count,addnode);

	   tmpnode = new DetailRecord();
	   index = parent.DetailInfo.length;
	   //Add record
	      parent.DetailInfo[index] = new DetailRecord();
		parent.DetailInfo[index].Version = addnode.Version;
	   	parent.DetailInfo[index].TestName = addnode.TestName;
		parent.DetailInfo[index].CGMCategory = addnode.CGMCategory;
		parent.DetailInfo[index].CGMElement = addnode.CGMElement;
		parent.DetailInfo[index].CatNote = addnode.CatNote;
		parent.DetailInfo[index].CGMPurpose = addnode.CGMPurpose;
		parent.DetailInfo[index].CheckPoints = addnode.CheckPoints;
		parent.DetailInfo[index].Note = addnode.Note;
	   //--------------------------------
	   //Clear testcase fields in addnode
	   //--------------------------------
	   addnode.TestName = "";
	   addnode.CGMPurpose = "";
	   addnode.CheckPoints = "";
	   addnode.Note = "";
	   break;
	}
	case "4-CGMPurpose":{
	   addnode.CGMPurpose = value;
	   break;
	}
        case "4-Note":{
	   addnode.Note = value
	   break;
	}
	case "4-CheckPoints":{
	   addnode.CheckPoints = "<TABLE><TD width=10><TD>";
	   BuildDetailInfo(list.item(i),count,addnode);
           addnode.CheckPoints = addnode.CheckPoints+"</TABLE>";
	   break;
	}
	case "5-CheckPoint":{
	   addnode.CheckPoints = addnode.CheckPoints + "<TR><TD colspan=2>"+value;
	   break;
	}
	case "5-SubPoint":{
	   addnode.CheckPoints = addnode.CheckPoints + "<TR><TD><TD>"+value;
	   break;
	}
	case "5-SubTable":{
	   addnode.CheckPoints = addnode.CheckPoints + "<TABLE BORDER>";
	   BuildDetailInfo(list.item(i),count,addnode);
	   addnode.CheckPoints = addnode.CheckPoints+"</TABLE>";
	   break;
	}
	case "6-Row":{
	   addnode.Checkpoints = addnode.Checkpoints + "<TR>";
	   BuildDetailInfo(list.item(i),count,addnode);
	   addnode.CheckPoints = addnode.CheckPoints+"</TR>";
	   break;
	}
	case "7-TD":{
	   addnode.CheckPoints = addnode.CheckPoints + "<TD align=center>"+value+"</TD>";
	   break;
	}
      default:{
      	   alert ("Unidentified Node: " + check);
      }
	}//end case
	list= HOLD;
   }
}
//******************************************************
//                     BuildCGMCategoryFileInfo
//
//  Passed:
//      Array to build from
//      Array to build to
//  Purpose:
//      Build the CGM Category array.  This array is used 
//          to build the category pulldown on the menu as
//          well as the summary report that is displayed
//          below the pulldowns.
//******************************************************
function BuildCGMCategoryFileInfo(DetailInfo,CGMCategoryInfo){
	for (var i=1;i<CGMCategoryInfo.length;i++){
	    CGMCategoryInfo[i].FileList = "";
	    lookingat = "";
	    min = 100;
	    max = 0;
	    Tmpmin = 10;
            Tmpmax = -10;
	    for (var j=0;j<DetailInfo.length;j++){
		if(DetailInfo[j].CGMCategory == CGMCategoryInfo[i].Value){
		   var FN = DetailInfo[j].TestName;
		   Type = FN.substr(0,6);
		   Num = FN.substr(6,2);
		   if (lookingat == "")
			lookingat = Type;
		   if (Type == lookingat){
			if (Num < min){ 
			   min = Num;
			   Tmpmin = Num;
			}
			if (Num > max){
			   max = Num;
			   Tmpmax = Num;
			}
		   }else{
			if (min == max)
			   CGMCategoryInfo[i].FileList = CGMCategoryInfo[i].FileList+"<BR>"+lookingat+Tmpmin;
			else
			   CGMCategoryInfo[i].FileList = CGMCategoryInfo[i].FileList+"<BR>"+lookingat+Tmpmin+"..."+Tmpmax;
			lookingat = Type;
			min = Num;
			tmpmin = Num;
			tmpmax = Num;
			max = Num;
		   }
		}
	     } 
	     if (min == max)
		CGMCategoryInfo[i].FileList = CGMCategoryInfo[i].FileList+"<BR>"+lookingat+Tmpmin;
	     else
		CGMCategoryInfo[i].FileList = CGMCategoryInfo[i].FileList+"<BR>"+lookingat+Tmpmin+"..."+Tmpmax;
	}
}

//******************************************************
//                 BuildPullDownInfo
//
//  Purpose:
//    Build the three pulldown menus
//  Passed:
//    array holding the information to add to the pull down menus.
//    the three array which hold the information for the 3 pulldowns.
//  Note:
//    only unique information should be listed in the pulldowns
//    the pulldowns should in in sorted order.
//******************************************************
function BuildPullDownInfo (DetailInfo, VersionInfo, CGMCategoryInfo,TestNameInfo){
	//Build All for each category
	VersionInfo[0] = new VersionRecord("All","Display All Versions");
	CGMCategoryInfo[0] = new CGMCategoryRecord ("All","Display All Categories");
	TestNameInfo[0] = new TestNameRecord ("All","Display All Test Names","All","All");

	tmpVersion = new VersionRecord();
	tmpCGMCategory = new CGMCategoryRecord();
	tmpTestName = new TestNameRecord();

	addVersion = new VersionRecord();
	addCGMCategory = new CGMCategoryRecord();
	addTestName = new TestNameRecord();

	for (var i=0;i<DetailInfo.length;i++){
	    //-----------------
	    //Work with Version
	    //-----------------
	    addVersion.Value = DetailInfo[i].Version;
	    addVersion.Description = "Display Version "+DetailInfo[i].Version+" test cases";
	    
	    found = 1;
   	    for (var j=1;j<VersionInfo.length;j++){
	   	if (addVersion.Value == VersionInfo[j].Value)
		   found = 0;
	   	if ((addVersion.Value < VersionInfo[j].Value) && (found)){
		   tempVersion = VersionInfo[j];
		   VersionInfo[j] = addVersion;
		   addVersion = tempVersion;
	      }
	    }
	    if(found)
	   	VersionInfo[VersionInfo.length] = new VersionRecord (addVersion.Value,addVersion.Description);
	    

	    //---------------------
	    //Work with CGMCategory
	    //---------------------
	    addCGMCategory.Value = DetailInfo[i].CGMCategory;
	    addCGMCategory.Description = "Display CGM Category "+DetailInfo[i].CGMCategory+" test cases";
	    addCGMCategory.CatNote = DetailInfo[i].CatNote;
	    addCGMCategory.CGMElement = DetailInfo[i].CGMElement;
	    found = 1;
   	    for (var j=1;j<CGMCategoryInfo.length;j++){
	   	if (addCGMCategory.Value == CGMCategoryInfo[j].Value)
		   found = 0;
	   	if ((addCGMCategory.Value < CGMCategoryInfo[j].Value) && (found)){
		   tempCGMCategory = CGMCategoryInfo[j];
		   CGMCategoryInfo[j] = addCGMCategory;
		   addCGMCategory = tempCGMCategory;
	   	}
	    }
	    
	    if(found)
		CGMCategoryInfo[CGMCategoryInfo.length] = new CGMCategoryRecord(addCGMCategory.Value,addCGMCategory.Description,addCGMCategory.CatNote,addCGMCategory.CGMElement);

	    //------------------
	    //Work with TestName
	    //------------------
	    addTestName.Value = DetailInfo[i].TestName;
	    addTestName.Description = "Display Test Case "+DetailInfo[i].TestName;
	    addTestName.CGMCategory = DetailInfo[i].CGMCategory;
	    addTestName.Version = DetailInfo[i].Version;
	    TestNameInfo[TestNameInfo.length] = new TestNameRecord(addTestName.Value,addTestName.Description,addTestName.CGMCategory,addTestName.Version);
	}
}

//******************************************************
//         DOMInitialSetup
//
//  Purpose:
//    set up the information array from the file
//    set up the 3 pulldown menu arrays.
//  Passed:
//    blank information array
//    3 blank arrays to be used for the pulldowns
//******************************************************
function DOMInitialSetup(DetailInfo, VersionInfo, CGMCategoryInfo,TestNameInfo){
	var xmlDocument = masterDoc1;
	count = 0;
	addnode = new DetailRecord();
	//CLEAR NON-REQUIRED FIELDS
	addnode.CGMPurpose = "";
	addnode.CheckPoints = "";
	addnode.Note = "";
	addnode.CatNote = "";
	addnode.CGMElement = "";
	
	BuildDetailInfo(xmlDocument.documentElement,count,addnode);
	SortDetailInfo(DetailInfo);
	BuildPullDownInfo(DetailInfo, VersionInfo, CGMCategoryInfo,TestNameInfo);
	BuildCGMCategoryFileInfo(DetailInfo,CGMCategoryInfo);
}

//******************************************************
//       DetailRecord
//
//  Purpose:
//    Define a detail record
//  Used for:
//    Detail information
//******************************************************
function DetailRecord(Version,CGMElement,CGMCategory,TestName,CGMPurpose,CatNote,checkpoints,Note){
	this.Version 		= Version;
	this.CGMElement 	= CGMElement;
	this.CGMCategory 	= CGMCategory;
	this.TestName 		= TestName;
	this.CGMPurpose		= CGMPurpose;
	this.CatNote		= CatNote;
	this.CheckPoints	= checkpoints;
	this.Note		= Note;
}
//******************************************************
//       VersionRecord
//
//  Purpose:
//    Define a Version record
//  Used for:
//    Version pulldown
//******************************************************
function VersionRecord (Value,Description){
	this.Value = Value;
	this.Description = Description;
}
//******************************************************
//       CGMCategoryRecord
//
//  Purpose:
//    Define a CGM Category record
//  Used for:
//    CGMCategory pulldown
//******************************************************
function CGMCategoryRecord (Value,Description,Note,CGMElement,FileList){
	this.Value = Value;
	this.Description = Description;
	this.CatNote = Note;
	this.FileList = FileList;
	this.CGMElement = CGMElement;
}
//******************************************************
//       TestNameRecord
//
//  Purpose:
//    Define a test name record
//  Used for:
//    Test Name pulldown
//******************************************************
function TestNameRecord (Value,Description,Category,Version){
	this.Value = Value;
	this.Description = Description;
	this.CGMCategory = Category;
	this.Version = Version;
}
