import { Accordion, AccordionItem, Stack, Text } from "@chakra-ui/react"
import { ProcessedRequest } from "declarations/b3_wallet/b3_wallet.did"
import { useEffect, useState } from "react"
import { B3BasicWallet, B3Wallet } from "service/actor"
import Processed from "./Processed"

interface ProcessedProps {
  setLoading: (loading: boolean) => void
  actor: B3Wallet | B3BasicWallet
}

const ProcessedList: React.FC<ProcessedProps> = ({ setLoading, actor }) => {
  const [processedList, setProcessedList] = useState<ProcessedRequest[]>([])

  useEffect(() => {
    console.log("fetching processed")
    setLoading(true)

    actor
      .get_processed_list()
      .then(processed => {
        console.log(processed)
        setProcessedList(processed.reverse())
        setLoading(false)
      })
      .catch(e => {
        console.log(e)
        setLoading(false)
      })
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [])

  return (
    <Stack spacing={4}>
      <Text fontSize="xl" fontWeight="bold">
        Processed
      </Text>
      <Accordion allowToggle>
        {processedList.map((request, i) => (
          <AccordionItem border="none" _focus={{ boxShadow: "none" }}>
            {({ isExpanded }) => (
              <Processed key={i} {...request} isExpanded={isExpanded} />
            )}
          </AccordionItem>
        ))}
      </Accordion>
    </Stack>
  )
}

export default ProcessedList
